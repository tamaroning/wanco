use anyhow::{bail, Result};
use inkwell::{
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue},
};

use crate::context::Context;

pub const MIGRATION_STATE_NONE: i32 = 0;
pub const MIGRATION_STATE_CHECKPOINT: i32 = 1;
pub const MIGRATION_STATE_RESTORE: i32 = 2;

pub fn gen_set_migration_state<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    migration_state: i32,
) -> Result<()> {
    let migration_state_ptr = ctx
        .builder
        .build_struct_gep(
            ctx.exec_env_type.unwrap(),
            *exec_env_ptr,
            *ctx.exec_env_fields.get("migration_state").unwrap(),
            "migration_state_ptr",
        )
        .expect("fail to build_struct_gep");
    let migration_state = ctx
        .inkwell_types
        .i32_type
        .const_int(migration_state as u64, false);
    ctx.builder
        .build_store(migration_state_ptr, migration_state)
        .expect("fail to build store");
    Ok(())
}

// Store the current stack frame if the migration state equals to MIAGRATION_STATE_CHECKPOINT
pub fn gen_check_state_and_snapshot<'a>(
    ctx: &mut Context<'a, '_>,
    current_fn: FunctionValue<'a>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    let then_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.then");
    let else_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.else");
    let cond = gen_compare_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_CHECKPOINT)
        .expect("fail to gen_compare_migration_state");
    ctx.builder
        .build_conditional_branch(cond.into_int_value(), then_bb, else_bb)
        .expect("should build conditional branch");
    ctx.builder.position_at_end(then_bb);
    gen_store_wasm_stack(ctx, exec_env_ptr, locals).expect("fail to gen_store_wasm_stack");
    gen_return_default_value(ctx, current_fn).expect("fail to gen_return_default_value");
    ctx.builder.position_at_end(else_bb);
    Ok(())
}

fn gen_compare_migration_state<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    migration_state: i32,
) -> Result<BasicValueEnum<'a>> {
    let migration_state_ptr = ctx
        .builder
        .build_struct_gep(
            ctx.exec_env_type.unwrap(),
            *exec_env_ptr,
            *ctx.exec_env_fields.get("migration_state").unwrap(),
            "migration_state_ptr",
        )
        .expect("fail to build_struct_gep");
    let current_migration_state = ctx
        .builder
        .build_load(
            ctx.inkwell_types.i32_type,
            migration_state_ptr,
            "current_migration_state",
        )
        .expect("fail to build load");
    let migration_state = ctx
        .inkwell_types
        .i32_type
        .const_int(migration_state as u64, false);
    let cmp = ctx
        .builder
        .build_int_compare(
            inkwell::IntPredicate::EQ,
            current_migration_state
                .as_basic_value_enum()
                .into_int_value(),
            migration_state.as_basic_value_enum().into_int_value(),
            "cmp_migration_state",
        )
        .expect("fail to build_int_compare");
    Ok(cmp.as_basic_value_enum())
}

fn gen_migration_state<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<BasicValueEnum<'a>> {
    let migration_state_ptr = unsafe {
        ctx.builder.build_struct_gep(
            ctx.inkwell_types.i32_type,
            *exec_env_ptr,
            *ctx.exec_env_fields.get("migration_state").unwrap(),
            "migration_state_ptr",
        )
    }
    .expect("fail to build_struct_gep");
    let migration_state = ctx
        .builder
        .build_load(
            ctx.inkwell_types.i32_type,
            migration_state_ptr,
            "migration_state",
        )
        .expect("fail to build load");
    Ok(migration_state)
}

fn gen_store_wasm_stack<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    // Store a frame
    ctx.builder
        .build_call(
            ctx.fn_new_frame.expect("should define new_frame"),
            &[exec_env_ptr.as_basic_value_enum().into()],
            "",
        )
        .expect("should build call");

    for (ptr, ty) in locals {
        let val = ctx
            .builder
            .build_load(ty.as_basic_type_enum(), *ptr, "")
            .expect("should build load");
        gen_add_local(ctx, exec_env_ptr, val).expect("should build add_local_T");
    }

    // Store stack values associated to the current function
    let frame = ctx.stack_frames.last().expect("frame empty");
    for _ in frame.stack.iter().rev() {
        // TODO:
    }
    Ok(())
}

fn gen_return_default_value<'a>(
    ctx: &mut Context<'a, '_>,
    current_fn: FunctionValue<'a>,
) -> Result<()> {
    let ret_type = current_fn.get_type().get_return_type();
    let Some(ty) = ret_type else {
        ctx.builder.build_return(None).expect("should build return");
        return Ok(());
    };
    match ty {
        BasicTypeEnum::IntType(ty) => {
            ctx.builder
                .build_return(Some(&ty.const_zero().as_basic_value_enum()))
                .expect("should build return");
        }
        BasicTypeEnum::FloatType(ty) => {
            ctx.builder
                .build_return(Some(&ty.const_zero().as_basic_value_enum()))
                .expect("should build return");
        }
        _ => unreachable!(),
    };
    Ok(())
}

fn gen_add_local<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    val: BasicValueEnum<'a>,
) -> Result<()> {
    if val.get_type().is_int_type() {
        if val.get_type().into_int_type() == ctx.inkwell_types.i32_type {
            ctx.builder
                .build_call(
                    ctx.fn_add_local_i32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else if val.get_type().into_int_type() == ctx.inkwell_types.i64_type {
            ctx.builder
                .build_call(
                    ctx.fn_add_local_i64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else {
            bail!("Unsupported type {:?}", val);
        }
    } else if val.get_type().is_float_type() {
        if val.get_type().into_float_type() == ctx.inkwell_types.f32_type {
            ctx.builder
                .build_call(
                    ctx.fn_add_local_f32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else if val.get_type().into_float_type() == ctx.inkwell_types.f64_type {
            ctx.builder
                .build_call(
                    ctx.fn_add_local_f64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else {
            bail!("Unsupported type {:?}", val);
        }
    } else if val.get_type().into_float_type() == ctx.inkwell_types.f64_type {
        ctx.builder
            .build_call(
                ctx.fn_add_local_f64.unwrap(),
                &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                "",
            )
            .expect("should build call");
    } else {
        bail!("Unsupported type {:?}", val);
    }
    Ok(())
}
