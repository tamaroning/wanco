pub mod stackmap;

use anyhow::Result;
use inkwell::{
    types::BasicTypeEnum,
    values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, PointerValue},
};

use crate::context::Context;

use super::cr::{gen_compare_migration_state, MIGRATION_STATE_CHECKPOINT_START};

pub fn gen_migration_point_v2<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<()> {
    let current_fn = ctx.current_fn.unwrap();
    let test = gen_compare_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_CHECKPOINT_START)
        .expect("fail to gen_compare_migration_state");

    let then_block = ctx
        .ictx
        .append_basic_block(current_fn, "migration_triggered");
    let else_block = ctx
        .ictx
        .append_basic_block(current_fn, "migration_not_triggered");
    ctx.builder
        .build_conditional_branch(test.into_int_value(), then_block, else_block)
        .expect("fail to build_conditional_branch");

    ctx.builder.position_at_end(then_block);
    // TODO:
    ctx.builder
        .build_unconditional_branch(else_block)
        .expect("fail to build_unconditional_branch");

    ctx.builder.position_at_end(else_block);
    Ok(())
}

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
    for (ptr, ty) in locals.iter() {
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
