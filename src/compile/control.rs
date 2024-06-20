use crate::context::Context;
use anyhow::{bail, Result};
use inkwell::{
    basic_block::BasicBlock,
    values::{BasicMetadataValueEnum, FunctionValue, PhiValue},
};
use wasmparser::{BlockType, BrTable};

use super::compile_type::wasmty_to_llvmty;

/// Holds the state of if-else.
#[derive(Eq, PartialEq, Debug)]
pub enum IfElseState {
    If,
    Else,
}

/// Holds the state of unreachable.
#[derive(Eq, PartialEq, Debug)]
pub enum UnreachableReason {
    Br,
    Return,
    Unreachable,
    Reachable,
}

impl UnreachableReason {
    fn is_jumped(&self) -> bool {
        match self {
            UnreachableReason::Br | UnreachableReason::Unreachable | UnreachableReason::Return => {
                true
            }
            UnreachableReason::Reachable => false,
        }
    }
}

/// Holds the state of control instructions: block, loop, if-else.
#[derive(Debug)]
pub enum ControlFrame<'a> {
    Loop {
        loop_body: BasicBlock<'a>,
        loop_next: BasicBlock<'a>,
        body_phis: Vec<PhiValue<'a>>,
        end_phis: Vec<PhiValue<'a>>,
        stack_size: usize,
    },
    Block {
        next: BasicBlock<'a>,
        end_phis: Vec<PhiValue<'a>>,
        stack_size: usize,
    },
    IfElse {
        if_then: BasicBlock<'a>,
        if_else: BasicBlock<'a>,
        if_end: BasicBlock<'a>,
        ifelse_state: IfElseState,
        end_phis: Vec<PhiValue<'a>>,
        stack_size: usize,
    },
}

impl<'a> ControlFrame<'a> {
    fn br_dest(&self) -> &BasicBlock<'a> {
        match self {
            ControlFrame::Loop { ref loop_body, .. } => loop_body,
            ControlFrame::Block { ref next, .. } => next,
            ControlFrame::IfElse { ref if_end, .. } => if_end,
        }
    }
}

pub fn gen_block(ctx: &mut Context<'_, '_>, blockty: BlockType) -> Result<()> {
    let current_block = ctx.builder.get_insert_block().unwrap();
    let next_block = ctx.ictx.append_basic_block(
        ctx.function_values[ctx.current_function_idx as usize],
        "block_next",
    );

    // Phi
    ctx.builder.position_at_end(next_block);
    let mut phis: Vec<PhiValue> = Vec::new();
    match blockty {
        BlockType::Empty => {}
        BlockType::Type(valty) => {
            ctx.builder.position_at_end(next_block);
            let phi = ctx
                .builder
                .build_phi(wasmty_to_llvmty(ctx, valty).unwrap(), "end_phi")
                .expect("should build phi");
            phis.push(phi);
        }
        BlockType::FuncType(..) => {
            unreachable!("Unexpected FuncType");
        }
    }

    ctx.builder.position_at_end(current_block);
    ctx.control_frames.push(ControlFrame::Block {
        next: next_block,
        end_phis: phis,
        stack_size: ctx.stack.len(),
    });
    Ok(())
}

pub fn gen_loop(ctx: &mut Context<'_, '_>, blockty: BlockType) -> Result<()> {
    let current_block = ctx.builder.get_insert_block().unwrap();

    // Create blocks
    let body_block = ctx.ictx.append_basic_block(
        ctx.function_values[ctx.current_function_idx as usize],
        "loop_body",
    );
    let next_block = ctx.ictx.append_basic_block(
        ctx.function_values[ctx.current_function_idx as usize],
        "loop_next",
    );

    // Phi
    ctx.builder.position_at_end(next_block);
    let body_phis: Vec<PhiValue> = Vec::new();
    let mut phis: Vec<PhiValue> = Vec::new();
    match blockty {
        BlockType::Empty => {}
        BlockType::Type(valty) => {
            ctx.builder.position_at_end(next_block);
            let phi = ctx
                .builder
                .build_phi(wasmty_to_llvmty(ctx, valty).unwrap(), "end_phi")
                .expect("should build phi");
            phis.push(phi);
        }
        BlockType::FuncType(..) => {
            unreachable!("Unexpected FuncType");
        }
    }

    ctx.control_frames.push(ControlFrame::Loop {
        loop_body: body_block,
        loop_next: next_block,
        body_phis,
        end_phis: phis,
        stack_size: ctx.stack.len(),
    });

    // Move to loop_body
    ctx.builder.position_at_end(current_block);
    ctx.builder
        .build_unconditional_branch(body_block)
        .expect("should build unconditional branch");
    ctx.builder.position_at_end(body_block);
    Ok(())
}

pub fn gen_if(ctx: &mut Context<'_, '_>, blockty: BlockType) -> Result<()> {
    let current_block = ctx
        .builder
        .get_insert_block()
        .expect("fail to get_insert_block");

    // Create blocks
    let then_block = ctx.ictx.append_basic_block(
        ctx.function_values[ctx.current_function_idx as usize],
        "then",
    );
    let else_block = ctx.ictx.append_basic_block(
        ctx.function_values[ctx.current_function_idx as usize],
        "else",
    );
    let end_block = ctx.ictx.append_basic_block(
        ctx.function_values[ctx.current_function_idx as usize],
        "end",
    );

    // Phi
    ctx.builder.position_at_end(end_block);
    let mut end_phis: Vec<PhiValue> = Vec::new();
    match blockty {
        BlockType::Empty => {}
        BlockType::Type(valty) => {
            ctx.builder.position_at_end(end_block);
            let phi = ctx
                .builder
                .build_phi(wasmty_to_llvmty(ctx, valty).unwrap(), "end_phi")
                .expect("should build phi");
            end_phis.push(phi);
        }
        BlockType::FuncType(..) => {
            bail!("Unexpected FuncType");
        }
    }

    // Reserve blocks
    ctx.builder.position_at_end(current_block);
    ctx.control_frames.push(ControlFrame::IfElse {
        if_then: then_block,
        if_else: else_block,
        if_end: end_block,
        ifelse_state: IfElseState::If,
        end_phis,
        stack_size: ctx.stack.len(),
    });

    // Compare stack value vs zero
    let cond_value = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::NE,
            ctx.stack.pop().expect("stack empty").into_int_value(),
            ctx.inkwell_types.i32_type.const_int(0, false),
            "",
        )
        .expect("should build int compare");
    ctx.builder
        .build_conditional_branch(cond_value, then_block, else_block)
        .expect("should build conditional branch");

    // Jump to then block
    ctx.builder.position_at_end(then_block);
    Ok(())
}

pub fn gen_else(ctx: &mut Context<'_, '_>) -> Result<()> {
    let current_block = ctx
        .builder
        .get_insert_block()
        .expect("fail to get_insert_block");

    // Phi
    let framelen = ctx.control_frames.len();
    let frame = &mut ctx.control_frames[framelen - 1];
    match frame {
        ControlFrame::IfElse {
            if_else,
            if_end,
            ifelse_state,
            end_phis,
            ..
        } => {
            *ifelse_state = IfElseState::Else;

            // Phi
            if ctx.unreachable_depth == 0 {
                // Phi
                for phi in end_phis {
                    let value = ctx.stack.pop().expect("stack empty");
                    phi.add_incoming(&[(&value, current_block)]);
                }
            }

            // Jump to merge block from current block
            if !ctx.unreachable_reason.is_jumped() {
                ctx.builder
                    .build_unconditional_branch(*if_end)
                    .expect("should build unconditional branch");
            }

            // Define else block
            ctx.builder.position_at_end(*if_else);
        }
        _ => {
            unreachable!("Op Else with another ControlFrame");
        }
    }
    Ok(())
}
pub fn gen_br(ctx: &mut Context<'_, '_>, relative_depth: u32) -> Result<()> {
    let current_block = ctx
        .builder
        .get_insert_block()
        .expect("error get_insert_block");
    let frame = &ctx.control_frames[ctx.control_frames.len() - 1 - relative_depth as usize];

    let phis = match frame {
        ControlFrame::Block { end_phis, .. } => end_phis,
        ControlFrame::IfElse { end_phis, .. } => end_phis,
        ControlFrame::Loop { body_phis, .. } => body_phis,
    };
    for phi in phis {
        let value = ctx.stack.pop().expect("stack empty");
        phi.add_incoming(&[(&value, current_block)]);
    }

    ctx.builder
        .build_unconditional_branch(*frame.br_dest())
        .expect("should build unconditional branch");
    ctx.unreachable_depth += 1;
    ctx.unreachable_reason = UnreachableReason::Br;
    Ok(())
}

pub fn gen_brif(ctx: &mut Context<'_, '_>, relative_depth: u32) -> Result<()> {
    let current_block = ctx
        .builder
        .get_insert_block()
        .expect("error get_insert_block");
    let frame = &ctx.control_frames[ctx.control_frames.len() - 1 - relative_depth as usize];

    // Branch condition: whether the top value of stack is not zero
    let cond = ctx.stack.pop().expect("stack empty");
    let cond_value = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::NE,
            cond.into_int_value(),
            ctx.inkwell_types.i32_type.const_int(0, false),
            "",
        )
        .expect("should build int compare");

    // Phi
    let phis = match frame {
        ControlFrame::Block { end_phis, .. } => end_phis,
        ControlFrame::IfElse { end_phis, .. } => end_phis,
        ControlFrame::Loop { body_phis, .. } => body_phis,
    };
    let values = ctx.peekn(phis.len()).expect("fail stack peekn");
    for (i, phi) in phis.iter().enumerate().rev() {
        let value = values[i];
        phi.add_incoming(&[(&value, current_block)]);
    }

    // Create else block
    let else_block = ctx.ictx.append_basic_block(
        ctx.function_values[ctx.current_function_idx as usize],
        "brif_else",
    );

    // Branch
    ctx.builder
        .build_conditional_branch(cond_value, *frame.br_dest(), else_block)
        .expect("should build conditional branch");
    ctx.builder.position_at_end(else_block);
    Ok(())
}

pub fn gen_br_table(ctx: &mut Context<'_, '_>, targets: BrTable) -> Result<()> {
    let current_block = ctx
        .builder
        .get_insert_block()
        .expect("error get_insert_block");
    let idx = ctx.stack.pop().expect("stack empty");

    // default frame
    let default = targets.default();
    let default_frame = &ctx.control_frames[ctx.control_frames.len() - 1 - default as usize];

    // Phi
    let phis = match default_frame {
        ControlFrame::Block { end_phis, .. } => end_phis,
        ControlFrame::IfElse { end_phis, .. } => end_phis,
        ControlFrame::Loop { body_phis, .. } => body_phis,
    };
    let values = ctx.peekn(phis.len()).expect("fail stack peekn");
    for (i, phi) in phis.iter().enumerate().rev() {
        let value = values[i];
        log::debug!("- add_incoming to {:?}", phi);
        phi.add_incoming(&[(&value, current_block)]);
    }

    // cases
    let mut cases: Vec<_> = Vec::new();
    for (i, depth) in targets.targets().enumerate() {
        let depth = depth.expect("fail to get depth");
        let dest = &ctx.control_frames[ctx.control_frames.len() - 1 - depth as usize];
        let intv = ctx.inkwell_types.i32_type.const_int(i as u64, false);
        cases.push((intv, *dest.br_dest()));
        let phis = match dest {
            ControlFrame::Block { end_phis, .. } => end_phis,
            ControlFrame::IfElse { end_phis, .. } => end_phis,
            ControlFrame::Loop { body_phis, .. } => body_phis,
        };
        let values = ctx.peekn(phis.len()).expect("fail stack peekn");
        for (i, phi) in phis.iter().enumerate().rev() {
            let value = values[i];
            phi.add_incoming(&[(&value, current_block)]);
            log::debug!("- add_incoming to {:?}", phi);
        }
    }
    // switch
    ctx.builder
        .build_switch(idx.into_int_value(), *default_frame.br_dest(), &cases)
        .expect("should build switch");
    ctx.unreachable_depth += 1;
    ctx.unreachable_reason = UnreachableReason::Br;
    Ok(())
}

pub fn gen_end<'a>(ctx: &mut Context<'a, '_>, current_fn: FunctionValue<'a>) -> Result<()> {
    let current_block = ctx
        .builder
        .get_insert_block()
        .expect("fail to get_insert_block");

    let frame = ctx.control_frames.pop().expect("control frame empty");

    if ctx.control_frames.is_empty() {
        // End of function
        match ctx.unreachable_reason {
            UnreachableReason::Unreachable | UnreachableReason::Return => {
                ctx.builder.position_at_end(*frame.br_dest());
                if current_fn.get_type().get_return_type().is_none() {
                    ctx.builder.build_return(None).expect("should build return");
                } else {
                    let ret_ty = current_fn
                        .get_type()
                        .get_return_type()
                        .expect("failed to get ret type");
                    let dummy = ret_ty.const_zero();
                    ctx.builder
                        .build_return(Some(&dummy))
                        .expect("should build return");
                }
            }
            UnreachableReason::Reachable | UnreachableReason::Br => {
                ctx.builder
                    .build_unconditional_branch(*frame.br_dest())
                    .expect("should build unconditional branch");
                ctx.builder.position_at_end(*frame.br_dest());
                if current_fn.get_type().get_return_type().is_none() {
                    ctx.builder.build_return(None).expect("should build return");
                } else {
                    let phis = match frame {
                        ControlFrame::Block { ref end_phis, .. } => end_phis,
                        _ => {
                            unreachable!("Unexpected ControlFrame")
                        }
                    };
                    assert!(!phis.is_empty());

                    // Collect Phi
                    if ctx.unreachable_reason == UnreachableReason::Reachable {
                        for phi in phis {
                            let value = ctx.stack.pop().expect("stack empty");
                            phi.add_incoming(&[(&value, current_block)]);
                        }
                    }

                    // Return value
                    // TODO: support multiple phis
                    let value = phis[0].as_basic_value();
                    ctx.builder
                        .build_return(Some(&value))
                        .expect("should build return");
                }
            }
        }
    } else {
        // End of Block/IfElse/Loop
        let (next, end_phis, stack_size) = match frame {
            ControlFrame::Loop {
                loop_next,
                end_phis,
                stack_size,
                ..
            } => (loop_next, end_phis, stack_size),
            ControlFrame::Block {
                next,
                end_phis,
                stack_size,
            } => (next, end_phis, stack_size),
            ControlFrame::IfElse {
                if_else,
                if_end,
                ifelse_state,
                end_phis,
                stack_size,
                ..
            } => {
                // Case Else block doesn't exist
                if ifelse_state == IfElseState::If {
                    ctx.builder.position_at_end(if_else);
                    ctx.builder
                        .build_unconditional_branch(if_end)
                        .expect("should build unconditional branch");
                }
                (if_end, end_phis, stack_size)
            }
        };
        if ctx.unreachable_reason == UnreachableReason::Reachable {
            // Collect Phi
            for phi in &end_phis {
                let value = ctx.stack.pop().expect("stack empty");
                phi.add_incoming(&[(&value, current_block)]);
            }
            // Jump
            ctx.builder.position_at_end(current_block);
            ctx.builder
                .build_unconditional_branch(next)
                .expect("should build unconditional branch");
        }

        ctx.builder.position_at_end(next);
        ctx.reset_stack(stack_size);

        // Phi
        for phi in &end_phis {
            if phi.count_incoming() == 0 {
                log::debug!("- no phi");
                let basic_ty = phi.as_basic_value().get_type();
                let placeholder_value = basic_ty.const_zero();
                ctx.stack.push(placeholder_value);
                phi.as_instruction().erase_from_basic_block();
            } else {
                log::debug!("- phi.incoming = {}", phi.count_incoming());
                let value = phi.as_basic_value();
                ctx.stack.push(value);
            }
        }
    }
    Ok(())
}

pub fn gen_call(ctx: &mut Context<'_, '_>, function_index: u32) -> Result<()> {
    let fn_called = ctx.function_values[function_index as usize];

    // collect args from stack
    let mut args: Vec<BasicMetadataValueEnum> = Vec::new();

    /*
    if fn_called.get_name().to_str() == anyhow::Result::Ok("print") {
        args.push(ctx.pop_and_load().into());

        let arg0 = ctx.pop_and_load();
        let memory_base_local = ctx
            .builder
            .build_load(
                ctx.inkwell_types.i8_ptr_type,
                ctx.global_memory_base
                    .expect("should define global_memory_base")
                    .as_pointer_value(),
                "memory_base_local",
            )
            .expect("should build load");
        let memory_base_int = ctx
            .builder
            .build_ptr_to_int(
                memory_base_local.into_pointer_value(),
                ctx.inkwell_types.i64_type,
                "memory_base_int",
            )
            .expect("should build ptr to int");
        let offset = arg0.into_int_value();
        let translated_address = ctx
            .builder
            .build_int_add(memory_base_int, offset, "transed_addr")
            .expect("should build int add");
        args.push(translated_address.into());
    } else
    */
    {
        for _ in 0..fn_called.count_params() {
            args.push(ctx.stack.pop().expect("stack empty").into());
        }
    }
    // call
    args.reverse();
    let call_site = ctx
        .builder
        .build_call(fn_called, &args[..], "")
        .expect("should build call");
    if call_site.try_as_basic_value().is_left() {
        ctx.stack.push(
            call_site
                .try_as_basic_value()
                .left()
                .expect("fail translate call_site"),
        );
    }
    Ok(())
}

pub fn gen_call_indirect(
    ctx: &mut Context<'_, '_>,
    type_index: u32,
    table_index: u32,
) -> Result<()> {
    // TODO: support larger
    assert_eq!(table_index, 0);

    // Load function pointer
    let idx = ctx.stack.pop().expect("stack empty").into_int_value();
    let dst_addr = unsafe {
        ctx.builder.build_gep(
            ctx.inkwell_types.i8_ptr_type,
            ctx.global_table
                .expect("should define global_table")
                .as_pointer_value(),
            &[idx],
            "dst_addr",
        )
    }
    .expect("should build gep");
    let fptr = ctx
        .builder
        .build_load(ctx.inkwell_types.i8_ptr_type, dst_addr, "fptr")
        .expect("should build load");

    // args
    let func_type = ctx.signatures[type_index as usize];
    let mut args: Vec<BasicMetadataValueEnum> = Vec::new();
    for _ in 0..func_type.get_param_types().len() {
        args.push(ctx.stack.pop().expect("stack empty").into());
    }

    // call and push result
    args.reverse();
    let call_site = ctx
        .builder
        .build_indirect_call(func_type, fptr.into_pointer_value(), &args, "call_site")
        .expect("should build indirect call");
    if call_site.try_as_basic_value().is_left() {
        ctx.stack.push(
            call_site
                .try_as_basic_value()
                .left()
                .expect("fail translate call_site"),
        );
    }
    Ok(())
}

pub fn gen_drop(ctx: &mut Context<'_, '_>) -> Result<()> {
    ctx.stack.pop().expect("stack empty");
    Ok(())
}

pub fn gen_return(ctx: &mut Context<'_, '_>, current_fn: FunctionValue<'_>) -> Result<()> {
    // Phi
    ctx.unreachable_depth += 1;
    ctx.unreachable_reason = UnreachableReason::Return;

    if current_fn.get_type().get_return_type().is_none() {
        ctx.builder.build_return(None).expect("should build return");
    } else {
        // Return value
        // TODO: support multiple phis
        let ret = ctx.stack.pop().expect("stack empty");
        ctx.builder
            .build_return(Some(&ret))
            .expect("should build return");
    }

    Ok(())
}

pub fn gen_select(ctx: &mut Context<'_, '_>) -> Result<()> {
    let v3 = ctx.stack.pop().expect("stack empty");
    let v2 = ctx.stack.pop().expect("stack empty");
    let v1 = ctx.stack.pop().expect("stack empty");
    let cond = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::NE,
            v3.into_int_value(),
            ctx.inkwell_types.i32_type.const_zero(),
            "",
        )
        .expect("should build int compare");
    let res = ctx
        .builder
        .build_select(cond, v1, v2, "")
        .expect("should build select");
    ctx.stack.push(res);
    Ok(())
}

pub fn gen_unreachable(ctx: &mut Context<'_, '_>) -> Result<()> {
    ctx.unreachable_depth += 1;
    ctx.unreachable_reason = UnreachableReason::Unreachable;
    ctx.builder
        .build_unreachable()
        .expect("should build unreachable");
    Ok(())
}
