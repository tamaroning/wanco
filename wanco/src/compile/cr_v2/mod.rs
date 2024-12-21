pub mod stackmap;

use anyhow::Result;
use inkwell::{
    types::BasicTypeEnum,
    values::{BasicMetadataValueEnum, BasicValue, PointerValue},
};

use crate::context::Context;

use super::cr::{gen_compare_migration_state, MIGRATION_STATE_CHECKPOINT_START};

pub fn gen_stackmap<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    // wasm params and locals
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    let mut stackmap_args: Vec<BasicMetadataValueEnum<'_>> = vec![];
    // args[0] = stackmap id
    stackmap_args.push(
        ctx.inkwell_types
            .i64_type
            .const_int(ctx.get_next_stackmap_id(), false)
            .into(),
    );
    // args[1] = 0
    stackmap_args.push(ctx.inkwell_types.i32_type.const_zero().into());
    // args[2..] = all live llvm registers (locals, params, value stack)
    // exec_env
    stackmap_args.push(exec_env_ptr.as_basic_value_enum().into());
    // locals and params
    for (ptr, _) in locals.iter() {
        stackmap_args.push(ptr.as_basic_value_enum().into());
    }
    // value stack
    let stack = ctx.stack_frames.last().expect("stack empty");
    for i in 0..stack.stack.len() {
        let value = stack.stack[i];
        stackmap_args.push(value.into());
    }
    ctx.builder
        .build_call(ctx.inkwell_intrs.experimental_stackmap, &stackmap_args, "")
        .expect("should build call");

    Ok(())
}
