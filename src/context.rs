use inkwell::{
    basic_block::BasicBlock,
    builder::Builder,
    context::Context as InkwellContext,
    module::Module,
    types::{BasicTypeEnum, FunctionType},
    values::{BasicValueEnum, FunctionValue, GlobalValue},
};

use crate::{
    compile::control::{ControlFrame, UnreachableReason},
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

pub struct Context<'a, 'b> {
    // Inkwell related
    pub ictx: &'a InkwellContext,
    pub module: &'b Module<'a>,
    pub builder: Builder<'a>,
    pub inkwell_types: InkwellTypes<'a>,
    pub inkwell_intrs: InkwellIntrinsics<'a>,

    // synthesized stuffs
    pub wanco_init_block: Option<BasicBlock<'a>>,
    pub wanco_main_block: Option<BasicBlock<'a>>,

    pub global_memory_size: Option<GlobalValue<'a>>,
    pub global_memory_base: Option<GlobalValue<'a>>,

    pub global_table: Option<GlobalValue<'a>>,

    // module info
    pub signatures: Vec<FunctionType<'a>>,
    /// List of (function, index)
    pub functions: Vec<(String, u32)>,
    pub function_values: Vec<FunctionValue<'a>>,
    pub num_functions: u32,
    pub start_function_idx: Option<u32>,

    pub num_imports: u32,

    pub globals: Vec<Global<'a>>,

    // compiler state
    pub current_function_idx: u32,
    pub control_frames: Vec<ControlFrame<'a>>,
    /// Wasm value stack for the current builder position
    pub stack: Vec<BasicValueEnum<'a>>,
    pub unreachable_depth: u32,
    pub unreachable_reason: UnreachableReason,
}

impl<'a> Context<'a, '_> {
    pub fn new<'b>(
        ictx: &'a InkwellContext,
        module: &'b Module<'a>,
        builder: Builder<'a>,
    ) -> Context<'a, 'b> {
        let (inkwell_types, inkwell_intrs) = init_inkwell(ictx, module);

        Context {
            ictx,
            module,
            builder,
            inkwell_types,
            inkwell_intrs,

            wanco_init_block: None,
            wanco_main_block: None,
            global_memory_size: None,
            global_memory_base: None,
            global_table: None,

            signatures: Vec::new(),
            functions: Vec::new(),
            function_values: Vec::new(),
            num_functions: 0,
            start_function_idx: None,
            num_imports: 0,
            globals: Vec::new(),

            current_function_idx: u32::MAX,
            control_frames: Vec::new(),
            stack: Vec::new(),
            unreachable_depth: 0,
            unreachable_reason: UnreachableReason::Reachable,
        }
    }

    /// Pop two values from the stack
    pub fn pop2(&mut self) -> (BasicValueEnum<'a>, BasicValueEnum<'a>) {
        let v2 = self.stack.pop().expect("stack empty");
        let v1 = self.stack.pop().expect("stack empty");
        (v1, v2)
    }

    /// Peek n values from the stack
    pub fn peekn(&self, n: usize) -> Result<&[BasicValueEnum<'a>]> {
        if self.stack.len() < n {
            bail!("stack length too short {} vs {}", self.stack.len(), n);
        }
        let index = self.stack.len() - n;
        Ok(&self.stack[index..])
    }

    /// Restore the stack to the specified size
    pub fn reset_stack(&mut self, stack_size: usize) {
        self.stack.truncate(stack_size);
    }

    /// Pop the stack and load the value if it is a pointer.
    pub fn pop_and_load(&mut self) -> BasicValueEnum<'a> {
        let pop = self.stack.pop().expect("stack empty");
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
}
