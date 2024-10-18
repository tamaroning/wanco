use anyhow::{bail, Result};
use inkwell::{
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum, PointerValue},
};

use crate::context::{Context, Global};

use super::{
    gen_compare_migration_state, gen_set_migration_state, MIGRATION_STATE_CHECKPOINT_CONTINUE,
    MIGRATION_STATE_CHECKPOINT_START,
};

pub(crate) fn gen_store_globals<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<()> {
    let current_fn = ctx.current_fn.unwrap();
    let then_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.then");
    let else_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.else");
    let cond = gen_compare_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_CHECKPOINT_CONTINUE)
        .expect("fail to gen_compare_migration_state");
    ctx.builder
        .build_conditional_branch(cond.into_int_value(), then_bb, else_bb)
        .expect("should build conditional branch");
    ctx.builder.position_at_end(then_bb);

    // add globals
    let mut globals = Vec::new();
    for global in &ctx.globals {
        let value = match global {
            Global::Const { value } => *value,
            Global::Mut { ptr, ty } => {
                let value = ctx
                    .builder
                    .build_load(*ty, ptr.as_pointer_value(), "")
                    .expect("should build load");
                value
            }
        };
        globals.push(value);
    }
    for value in globals {
        gen_push_global_value(ctx, exec_env_ptr, value)
            .expect("should build push_global for const global");
    }
    ctx.builder
        .build_unconditional_branch(else_bb)
        .expect("should build unconditonal branch");
    // Move back to else bb
    ctx.builder.position_at_end(else_bb);
    Ok(())
}

fn gen_push_global_value<'a>(
    ctx: &Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    value: BasicValueEnum<'a>,
) -> Result<()> {
    if value.get_type().is_int_type() {
        if value.get_type().into_int_type() == ctx.inkwell_types.i32_type {
            ctx.builder
                .build_call(
                    ctx.fn_push_global_i32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), value.into()],
                    "",
                )
                .expect("should build call");
        } else if value.get_type().into_int_type() == ctx.inkwell_types.i64_type {
            ctx.builder
                .build_call(
                    ctx.fn_push_global_i64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), value.into()],
                    "",
                )
                .expect("should build call");
        } else {
            bail!("Unsupported type {:?}", value);
        }
    } else if value.get_type().is_float_type() {
        if value.get_type().into_float_type() == ctx.inkwell_types.f32_type {
            ctx.builder
                .build_call(
                    ctx.fn_push_global_f32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), value.into()],
                    "",
                )
                .expect("should build call");
        } else if value.get_type().into_float_type() == ctx.inkwell_types.f64_type {
            ctx.builder
                .build_call(
                    ctx.fn_push_global_f64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), value.into()],
                    "",
                )
                .expect("should build call");
        } else {
            bail!("Unsupported type {:?}", value);
        }
    } else {
        bail!("Unsupported type {:?}", value);
    }
    Ok(())
}

pub(crate) fn gen_store_table<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<()> {
    let Some(global_table) = ctx.global_table else {
        return Ok(());
    };
    let current_fn = ctx.current_fn.unwrap();
    let then_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.then");
    let else_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.else");
    let cond = gen_compare_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_CHECKPOINT_CONTINUE)
        .expect("fail to gen_compare_migration_state");
    ctx.builder
        .build_conditional_branch(cond.into_int_value(), then_bb, else_bb)
        .expect("should build conditional branch");
    ctx.builder.position_at_end(then_bb);

    for i in 0..ctx.global_table_size.unwrap() {
        let elem_ptr = unsafe {
            ctx.builder.build_gep(
                ctx.inkwell_types.i32_type,
                global_table.as_pointer_value(),
                &[ctx.ictx.i32_type().const_int(i as u64, false)],
                "fnidx_ptr",
            )
        }
        .expect("should build gep");
        let fnidx = ctx
            .builder
            .build_load(ctx.inkwell_types.i32_type, elem_ptr, "fnidx")
            .expect("should build load");
        ctx.builder
            .build_call(
                ctx.fn_push_table_index.unwrap(),
                &[exec_env_ptr.as_basic_value_enum().into(), fnidx.into()],
                "",
            )
            .expect("should build call");
    }

    ctx.builder
        .build_unconditional_branch(else_bb)
        .expect("should build unconditonal branch");
    // Move back to else bb
    ctx.builder.position_at_end(else_bb);
    Ok(())
}

pub(crate) fn gen_checkpoint_start<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    gen_set_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_CHECKPOINT_CONTINUE)
        .expect("fail to gen_set_migration_state");
    gen_store_frame(ctx, exec_env_ptr, locals).expect("fail to gen_store_frame");
    gen_store_stack(ctx, exec_env_ptr).expect("fail to gen_store_stack");
    gen_return_default_value(ctx).expect("fail to gen_return_default_value");
    Ok(())
}

pub fn gen_checkpoint_unwind<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    let current_fn = ctx.current_fn.expect("should define current_fn");
    let then_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.then");
    let else_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.else");
    let cond = gen_compare_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_CHECKPOINT_CONTINUE)
        .expect("fail to gen_compare_migration_state");
    ctx.builder
        .build_conditional_branch(cond.into_int_value(), then_bb, else_bb)
        .expect("should build conditional branch");
    ctx.builder.position_at_end(then_bb);
    gen_store_frame(ctx, exec_env_ptr, locals).expect("fail to gen_store_frame");
    gen_store_stack(ctx, exec_env_ptr).expect("fail to gen_store_stack");
    gen_return_default_value(ctx).expect("fail to gen_return_default_value");
    ctx.builder.position_at_end(else_bb);
    Ok(())
}

fn gen_store_frame<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    // Store a frame
    ctx.builder
        .build_call(
            ctx.fn_push_frame.expect("should define push_frame"),
            &[exec_env_ptr.as_basic_value_enum().into()],
            "",
        )
        .expect("should build call");
    let pc = ctx.current_op.unwrap() as u64;
    let fn_index = ctx.current_function_idx.unwrap() as u64;
    ctx.builder
        .build_call(
            ctx.fn_set_pc_to_frame.unwrap(),
            &[
                exec_env_ptr.as_basic_value_enum().into(),
                ctx.inkwell_types.i32_type.const_int(fn_index, false).into(),
                ctx.inkwell_types.i32_type.const_int(pc, false).into(),
            ],
            "",
        )
        .expect("should build call");

    for (ptr, ty) in locals {
        let val = ctx
            .builder
            .build_load(ty.as_basic_type_enum(), *ptr, "")
            .expect("should build load");
        gen_add_local(ctx, exec_env_ptr, val).expect("should build push_local_T");
    }
    Ok(())
}

fn gen_store_stack<'a>(ctx: &mut Context<'a, '_>, exec_env_ptr: &PointerValue<'a>) -> Result<()> {
    // Store stack values associated to the current function
    let frame = ctx.stack_frames.last().expect("frame empty");
    let stack = frame.stack.clone();
    for value in stack.iter().rev() {
        gen_push_stack(ctx, exec_env_ptr, *value).expect("should build push_T");
    }
    Ok(())
}

fn gen_return_default_value(ctx: &mut Context<'_, '_>) -> Result<()> {
    let ret_type = ctx.current_fn.unwrap().get_type().get_return_type();
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
                    ctx.fn_push_local_i32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else if val.get_type().into_int_type() == ctx.inkwell_types.i64_type {
            ctx.builder
                .build_call(
                    ctx.fn_push_local_i64.unwrap(),
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
                    ctx.fn_push_local_f32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else if val.get_type().into_float_type() == ctx.inkwell_types.f64_type {
            ctx.builder
                .build_call(
                    ctx.fn_push_local_f64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else {
            bail!("Unsupported type {:?}", val);
        }
    } else {
        bail!("Unsupported type {:?}", val);
    }
    Ok(())
}

fn gen_push_stack<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    val: BasicValueEnum<'a>,
) -> Result<()> {
    if val.get_type().is_int_type() {
        if val.get_type().into_int_type() == ctx.inkwell_types.i32_type {
            ctx.builder
                .build_call(
                    ctx.fn_push_i32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else if val.get_type().into_int_type() == ctx.inkwell_types.i64_type {
            ctx.builder
                .build_call(
                    ctx.fn_push_i64.unwrap(),
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
                    ctx.fn_push_f32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else if val.get_type().into_float_type() == ctx.inkwell_types.f64_type {
            ctx.builder
                .build_call(
                    ctx.fn_push_f64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into(), val.into()],
                    "",
                )
                .expect("should build call");
        } else {
            bail!("Unsupported type {:?}", val);
        }
    } else {
        bail!("Unsupported type {:?}", val);
    }
    Ok(())
}
