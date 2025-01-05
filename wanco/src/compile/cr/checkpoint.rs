use anyhow::{bail, Result};
use inkwell::{
    module::Linkage,
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum, PointerValue},
    AddressSpace,
};

use crate::context::{Context, Global};

use super::{
    gen_set_migration_state, MAX_LOCALS_STORE, MAX_STACK_STORE, MIGRATION_STATE_CHECKPOINT_CONTINUE,
};

pub(crate) fn add_fn_store_globals_api(ctx: &mut Context<'_, '_>) -> Result<()> {
    let fn_type = ctx.inkwell_types.void_type.fn_type(
        &[ctx
            .exec_env_type
            .unwrap()
            .ptr_type(AddressSpace::default())
            .into()],
        false,
    );
    let fn_store_globals =
        ctx.module
            .add_function("store_globals", fn_type, Some(Linkage::External));
    let entry_bb = ctx.ictx.append_basic_block(fn_store_globals, "entry");
    ctx.builder.position_at_end(entry_bb);
    let exec_env_ptr = fn_store_globals
        .get_nth_param(0)
        .unwrap()
        .into_pointer_value();
    gen_store_globals(ctx, &exec_env_ptr)?;

    ctx.builder.build_return(None).expect("should build return");

    Ok(())
}

fn gen_store_globals<'a>(ctx: &mut Context<'a, '_>, exec_env_ptr: &PointerValue<'a>) -> Result<()> {
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

pub(crate) fn add_fn_store_table_api(ctx: &mut Context<'_, '_>) -> Result<()> {
    let fn_type = ctx.inkwell_types.void_type.fn_type(
        &[ctx
            .exec_env_type
            .unwrap()
            .ptr_type(AddressSpace::default())
            .into()],
        false,
    );
    let fn_store_table = ctx
        .module
        .add_function("store_table", fn_type, Some(Linkage::External));
    let entry_bb = ctx.ictx.append_basic_block(fn_store_table, "entry");
    ctx.builder.position_at_end(entry_bb);
    let exec_env_ptr = fn_store_table
        .get_nth_param(0)
        .unwrap()
        .into_pointer_value();
    gen_store_table(ctx, &exec_env_ptr)?;

    ctx.builder.build_return(None).expect("should build return");

    Ok(())
}

fn gen_store_table<'a>(ctx: &mut Context<'a, '_>, exec_env_ptr: &PointerValue<'a>) -> Result<()> {
    let Some(global_table) = ctx.global_table else {
        return Ok(());
    };
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

fn gen_store_frame<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    let nlocals = locals.len();
    let nstack = ctx.stack_frames.last().unwrap().stack.len();
    if nlocals > MAX_LOCALS_STORE || nstack > MAX_STACK_STORE {
        log::warn!("Too large frame to checkpoint/restore, skipped");
        log::warn!("nlocals: {}, nstack: {}", nlocals, nstack);
        return Ok(());
    }

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
