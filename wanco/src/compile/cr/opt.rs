use std::collections::{HashMap, HashSet};

use wasmparser::{FunctionBody, Operator};

use crate::context::Context;
use anyhow::{anyhow, bail, Context as _, Result};

pub struct Analysis {
    num_function_imports: u32,
    callgraph: HashMap<u32, Vec<u32>>,
    // reverse callgraph
    rev_callgraph: HashMap<u32, Vec<u32>>,
    cycles: Vec<Vec<u32>>,
    cycle_edges: HashSet<(u32, u32)>,
    cycle_fn: HashSet<u32>,
    fn_has_loop: HashSet<u32>,
    //fn_has_call_indirect: HashSet<u32>,
}

impl Analysis {
    pub fn call_requires_migration_point(&self, caller: u32, callee: u32) -> bool {
        let is_external_function_call = callee < self.num_function_imports;
        self.cycle_edges.contains(&(caller, callee)) || is_external_function_call
    }

    pub fn call_indirect_requires_migration_point(&self, _: u32) -> bool {
        true
    }
}

pub fn run_analysis_pass(ctx: &mut Context, functions: &Vec<FunctionBody>) -> Result<()> {
    let mut analysis = Analysis {
        num_function_imports: ctx.num_imports,
        callgraph: HashMap::new(),
        rev_callgraph: HashMap::new(),
        cycles: vec![],
        cycle_edges: HashSet::new(),
        cycle_fn: HashSet::new(),
        fn_has_loop: HashSet::new(),
        //fn_has_call_indirect: HashSet::new(),
    };

    calculate_callgraph(ctx, &functions, &mut analysis)?;

    calulate_cycles(ctx, &mut analysis);

    calculate_has_loop(ctx, &functions, &mut analysis)?;

    //calculate_has_call_indirect(ctx, &functions, &mut analysis)?;

    // print! callgraph as graphviz
    println!("digraph callgraph {{");
    for (caller, callees) in analysis.callgraph.iter() {
        for callee in callees {
            // if caller->callee is in cycle, color the edge red
            if analysis.cycle_edges.contains(&(*caller, *callee)) {
                println!("  {} -> {} [color=red];", caller, callee);
            } else {
                println!("  {} -> {};", caller, callee);
            }
        }
    }
    // color the function node if it may take infinite time
    for func in ctx.num_imports..ctx.num_imports + functions.len() as u32 {
        if may_take_infinite_time(&analysis, func) {
            println!("  {} [color=blue];", func);
        }
    }
    println!("}}");

    ctx.analysis_v1 = Some(analysis);
    Ok(())
}

fn calculate_callgraph(
    ctx: &Context,
    functions: &Vec<FunctionBody>,
    analysis: &mut Analysis,
) -> Result<()> {
    let mut fn_index: u32 = ctx.num_imports;
    for func in functions {
        let callee = get_callee(func)?;

        // set reverse callgrph
        for c in &callee {
            let callees = analysis.rev_callgraph.entry(*c).or_insert(vec![]);
            callees.push(fn_index);
        }

        // set callgraph
        analysis.callgraph.insert(fn_index, callee);
        fn_index += 1;
    }
    Ok(())
}

fn calulate_cycles(ctx: &Context, analysis: &mut Analysis) {
    let start_fn_idx = ctx.start_function_idx.unwrap();
    let mut stack = vec![];
    find_cycle_dfs(ctx, analysis, &mut stack, start_fn_idx);
    compute_cycle_edges(analysis);
    compute_cycle_fn(analysis);
}

fn find_cycle_dfs(ctx: &Context, analysis: &mut Analysis, stack: &mut Vec<u32>, fn_index: u32) {
    if let Some((i, _)) = stack.iter().enumerate().find(|(i, f)| **f == fn_index) {
        let cycle = stack[i..].to_vec();
        analysis.cycles.push(cycle);
        return;
    }

    if let Some(callees) = analysis.callgraph.get(&fn_index) {
        for callee in callees.clone() {
            stack.push(callee);
            find_cycle_dfs(ctx, analysis, stack, callee);
            stack.pop();
        }
    }
}

fn compute_cycle_edges(analysis: &mut Analysis) {
    for cycle in analysis.cycles.iter() {
        for i in 0..cycle.len() {
            let j = (i + 1) % cycle.len();
            analysis.cycle_edges.insert((cycle[i], cycle[j]));
        }
    }
}

fn compute_cycle_fn(analysis: &mut Analysis) {
    for cycle in analysis.cycles.iter() {
        for fn_index in cycle {
            analysis.cycle_fn.insert(*fn_index);
        }
    }
}

fn calculate_has_loop(
    ctx: &Context,
    functions: &Vec<FunctionBody>,
    analysis: &mut Analysis,
) -> Result<()> {
    let mut fn_index: u32 = ctx.num_imports;
    for func in functions {
        let has_loop = has_loop(ctx, func)?;
        if has_loop {
            analysis.fn_has_loop.insert(fn_index);
        }
        fn_index += 1;
    }
    Ok(())
}

fn may_take_infinite_time(analysis: &Analysis, fn_index: u32) -> bool {
    return analysis.cycle_fn.contains(&fn_index) || analysis.fn_has_loop.contains(&fn_index);
}

fn is_external_function(ctx: &mut Context, fn_index: usize) -> bool {
    return fn_index < ctx.num_imports as usize;
}

fn has_loop(ctx: &Context, f: &FunctionBody) -> Result<bool> {
    let mut reader = f.get_operators_reader()?.get_binary_reader();
    while !reader.eof() {
        let op = reader.read_operator()?;
        match op {
            Operator::Loop { .. } => return Ok(true),
            _ => {}
        }
    }
    Ok(false)
}

/*
fn has_call_indirect(ctx: &Context, f: &FunctionBody) -> Result<bool> {
    let mut reader = f.get_operators_reader()?.get_binary_reader();
    while !reader.eof() {
        let op = reader.read_operator()?;
        match op {
            Operator::CallIndirect { .. } => return Ok(true),
            _ => {}
        }
    }
    Ok(false)
}

fn calculate_has_call_indirect(
    ctx: &Context,
    functions: &Vec<FunctionBody>,
    analysis: &mut Analysis,
) -> Result<()> {
    let mut fn_index: u32 = ctx.num_imports;
    for func in functions {
        let has_call_indirect = has_call_indirect(ctx, func)?;
        if has_call_indirect {
            analysis.fn_has_call_indirect.insert(fn_index);
        }
        fn_index += 1;
    }
    Ok(())
}
    */

fn get_callee(f: &FunctionBody) -> Result<Vec<u32>> {
    let mut callee = vec![];

    let mut reader = f.get_operators_reader()?.get_binary_reader();
    while !reader.eof() {
        let op = reader.read_operator()?;
        match op {
            Operator::Call { function_index } => {
                callee.push(function_index);
            }
            /*
            Operator::CallIndirect {
                type_index: _,
                table_index: _,
            } => {
                callee.push(123456);
            }
            */
            _ => {}
        }
    }
    // remove dup
    callee.sort();
    callee.dedup();
    Ok(callee)
}
