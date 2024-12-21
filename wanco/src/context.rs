use std::{collections::HashMap, sync::atomic::AtomicU64};

use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context as InkwellContext,
    module::Module,
    types::{BasicTypeEnum, FunctionType, StructType},
    values::{BasicValueEnum, FunctionValue, GlobalValue, IntValue},
};

use crate::{
    compile::control::{ControlFrame, UnreachableReason},
    driver::Args,
    inkwell::{init_inkwell, InkwellIntrinsics, InkwellTypes},
};

use anyhow::{bail, Result};

pub enum Global<'a> {
    Mut {
        /// Pointer to the actual global value
        ptr: GlobalValue<'a>,
        ty: BasicTypeEnum<'a>,
    },
    Const {
        value: BasicValueEnum<'a>,
    },
}

pub struct StackFrame<'a> {
    pub stack: Vec<BasicValueEnum<'a>>,
}

impl<'a> StackFrame<'a> {
    pub fn new() -> StackFrame<'a> {
        StackFrame { stack: Vec::new() }
    }
}

pub struct Function {
    pub name: String,
    pub type_idx: u32,
    // (module, name)
    pub orig_name: Option<(String, String)>,
}

pub struct Context<'a, 'b> {
    pub config: &'a Args,

    // Inkwell related
    pub ictx: &'a InkwellContext,
    pub module: &'b Module<'a>,
    pub builder: Builder<'a>,
    pub inkwell_types: InkwellTypes<'a>,
    pub inkwell_intrs: InkwellIntrinsics<'a>,

    // synthesized stuffs
    /// aot_main %init block
    pub aot_init_block: Option<BasicBlock<'a>>,
    /// aot_main %main block
    pub aot_main_block: Option<BasicBlock<'a>>,

    pub fn_memory_grow: Option<FunctionValue<'a>>,
    pub global_table: Option<GlobalValue<'a>>,
    pub global_table_size: Option<usize>,
    pub global_fptr_array: Option<GlobalValue<'a>>,

    pub exec_env_type: Option<StructType<'a>>,
    pub exec_env_fields: HashMap<&'static str, u32>,

    // module info
    pub signatures: Vec<FunctionType<'a>>,
    /// List of (function, index)
    pub functions: Vec<Function>,
    pub function_values: Vec<FunctionValue<'a>>,
    pub num_functions: u32,
    pub start_function_idx: Option<u32>,

    pub num_imports: u32,

    pub globals: Vec<Global<'a>>,

    // builder state
    pub current_function_idx: Option<u32>,
    pub current_fn: Option<FunctionValue<'a>>,
    pub current_op: Option<u32>,
    pub control_frames: Vec<ControlFrame<'a>>,
    /// Wasm value stack for the current builder position
    pub stack_frames: Vec<StackFrame<'a>>,
    pub unreachable_depth: u32,
    pub unreachable_reason: UnreachableReason,

    // checkpoint related
    //pub exception_type: StructType<'a>,
    //pub personality_function: FunctionValue<'a>,
    pub fn_push_frame: Option<FunctionValue<'a>>,
    pub fn_set_pc_to_frame: Option<FunctionValue<'a>>,
    pub fn_push_local_i32: Option<FunctionValue<'a>>,
    pub fn_push_local_i64: Option<FunctionValue<'a>>,
    pub fn_push_local_f32: Option<FunctionValue<'a>>,
    pub fn_push_local_f64: Option<FunctionValue<'a>>,
    pub fn_push_i32: Option<FunctionValue<'a>>,
    pub fn_push_i64: Option<FunctionValue<'a>>,
    pub fn_push_f32: Option<FunctionValue<'a>>,
    pub fn_push_f64: Option<FunctionValue<'a>>,
    pub fn_push_global_i32: Option<FunctionValue<'a>>,
    pub fn_push_global_i64: Option<FunctionValue<'a>>,
    pub fn_push_global_f32: Option<FunctionValue<'a>>,
    pub fn_push_global_f64: Option<FunctionValue<'a>>,
    pub fn_push_table_index: Option<FunctionValue<'a>>,
    pub fn_start_checkpoint: Option<FunctionValue<'a>>,
    // restore related
    pub fn_pop_front_frame: Option<FunctionValue<'a>>,
    pub fn_frame_is_empty: Option<FunctionValue<'a>>,
    pub fn_get_pc_from_frame: Option<FunctionValue<'a>>,
    pub fn_pop_front_local_i32: Option<FunctionValue<'a>>,
    pub fn_pop_front_local_i64: Option<FunctionValue<'a>>,
    pub fn_pop_front_local_f32: Option<FunctionValue<'a>>,
    pub fn_pop_front_local_f64: Option<FunctionValue<'a>>,
    pub fn_pop_i32: Option<FunctionValue<'a>>,
    pub fn_pop_i64: Option<FunctionValue<'a>>,
    pub fn_pop_f32: Option<FunctionValue<'a>>,
    pub fn_pop_f64: Option<FunctionValue<'a>>,
    pub fn_pop_front_global_i32: Option<FunctionValue<'a>>,
    pub fn_pop_front_global_i64: Option<FunctionValue<'a>>,
    pub fn_pop_front_global_f32: Option<FunctionValue<'a>>,
    pub fn_pop_front_global_f64: Option<FunctionValue<'a>>,
    pub fn_pop_front_table_index: Option<FunctionValue<'a>>,

    // restore related builder state
    pub restore_dispatch_bb: Option<BasicBlock<'a>>,
    pub restore_dispatch_cases: Vec<(IntValue<'a>, BasicBlock<'a>)>,

    pub analysis_v1: Option<crate::compile::cr::opt::Analysis>,

    // C/R v2
    next_stackmap_id: AtomicU64,

    // common to both C/R v1 and v2
    pub num_migration_points: u32,
}

impl<'a> Context<'a, '_> {
    pub fn new<'b>(
        args: &'a Args,
        ictx: &'a InkwellContext,
        module: &'b Module<'a>,
        builder: Builder<'a>,
    ) -> Context<'a, 'b> {
        let (inkwell_types, inkwell_intrs) = init_inkwell(ictx, module);

        Context {
            config: args,
            ictx,
            module,
            builder,
            inkwell_types,
            inkwell_intrs,

            aot_init_block: None,
            aot_main_block: None,
            fn_memory_grow: None,
            exec_env_type: None,
            exec_env_fields: HashMap::new(),
            global_fptr_array: None,
            global_table: None,
            global_table_size: None,

            signatures: Vec::new(),
            functions: Vec::new(),
            function_values: Vec::new(),
            num_functions: 0,
            start_function_idx: None,
            num_imports: 0,
            globals: Vec::new(),

            current_function_idx: None,
            current_fn: None,
            current_op: None,
            control_frames: Vec::new(),
            stack_frames: Vec::new(),
            unreachable_depth: 0,
            unreachable_reason: UnreachableReason::Reachable,

            fn_push_frame: None,
            fn_set_pc_to_frame: None,
            fn_push_local_i32: None,
            fn_push_local_i64: None,
            fn_push_local_f32: None,
            fn_push_local_f64: None,
            fn_push_i32: None,
            fn_push_i64: None,
            fn_push_f32: None,
            fn_push_f64: None,
            fn_push_global_i64: None,
            fn_push_global_i32: None,
            fn_push_global_f32: None,
            fn_push_global_f64: None,
            fn_push_table_index: None,
            fn_start_checkpoint: None,

            fn_get_pc_from_frame: None,
            fn_frame_is_empty: None,
            fn_pop_front_frame: None,
            fn_pop_front_local_i32: None,
            fn_pop_front_local_i64: None,
            fn_pop_front_local_f32: None,
            fn_pop_front_local_f64: None,
            fn_pop_i32: None,
            fn_pop_i64: None,
            fn_pop_f32: None,
            fn_pop_f64: None,
            fn_pop_front_global_i32: None,
            fn_pop_front_global_i64: None,
            fn_pop_front_global_f32: None,
            fn_pop_front_global_f64: None,
            fn_pop_front_table_index: None,

            restore_dispatch_bb: None,
            restore_dispatch_cases: Vec::new(),

            analysis_v1: None,

            next_stackmap_id: AtomicU64::new(0),

            num_migration_points: 0,
        }
    }

    /// Push a value to the current stack frame
    pub fn push(&mut self, value: BasicValueEnum<'a>) {
        let frame = self.stack_frames.last_mut().expect("frame empty");
        frame.stack.push(value);
    }

    /// Pop a value from the current stack frame
    pub fn pop(&mut self) -> Option<BasicValueEnum<'a>> {
        let frame = self.stack_frames.last_mut().expect("frame empty");
        frame.stack.pop()
    }

    /// Pop two values from the current stack frame
    pub fn pop2(&mut self) -> (BasicValueEnum<'a>, BasicValueEnum<'a>) {
        let frame = self.stack_frames.last_mut().expect("frame empty");
        let v2 = frame.stack.pop().expect("stack empty");
        let v1 = frame.stack.pop().expect("stack empty");
        (v1, v2)
    }

    /// Peek n values from the stack bottom
    pub fn peekn(&self, n: usize) -> Result<&[BasicValueEnum<'a>]> {
        let frame = self.stack_frames.last().expect("frame empty");
        if frame.stack.len() < n {
            bail!(
                "stack length too short. Expected {} but found {}",
                n,
                frame.stack.len()
            );
        }
        let index = frame.stack.len() - n;
        Ok(&frame.stack[index..])
    }

    /// Peek nth value from the stack top (top is 0th value)
    pub fn peek_from_top(&self, n: usize) -> Result<&BasicValueEnum<'a>> {
        let frame = self.stack_frames.last().expect("frame empty");
        let mut it = frame.stack.iter().rev().skip(n);
        it.next()
            .ok_or_else(|| anyhow::anyhow!("stack length too short"))
    }

    /// Get size of the current stack frame
    pub fn current_frame_size(&self) -> usize {
        let frame = self.stack_frames.last().expect("frame empty");
        frame.stack.len()
    }

    /// Restore the current stack frame to the specified size
    pub fn reset_stack(&mut self, stack_size: usize) {
        let frame = self.stack_frames.last_mut().expect("frame empty");
        frame.stack.truncate(stack_size);
    }

    /// Pop the stack and load the value if it is a pointer.
    /*
    pub fn pop_and_load(&mut self) -> BasicValueEnum<'a> {
        let frame = self.stack_frames.last_mut().expect("frame empty");
        let pop = frame.stack.pop().expect("stack empty");
        if pop.is_pointer_value() {
            self.builder
                .build_load(
                    self.inkwell_types.i64_type,
                    pop.into_pointer_value(),
                    "from_stack",
                )
                .expect("load from stack")
        } else {
            pop
        }
    }
    */

    pub fn get_next_stackmap_id(&self) -> u64 {
        let id = self
            .next_stackmap_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        id
    }
}
