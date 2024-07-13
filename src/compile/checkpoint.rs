use anyhow::{bail, Result};
use inkwell::{
    basic_block::BasicBlock,
    types::{BasicType, BasicTypeEnum},
    values::{AnyValue, BasicValue, BasicValueEnum, CallSiteValue, FunctionValue, PointerValue},
};

use crate::context::{Context, Global};

pub const MIGRATION_STATE_NONE: i32 = 0;
pub const MIGRATION_STATE_CHECKPOINT_START: i32 = 1;
pub const MIGRATION_STATE_CHECKPOINT_CONTINUE: i32 = 2;
pub const MIGRATION_STATE_RESTORE: i32 = 3;

pub fn gen_restore_dispatch<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<()> {
    let current_fn = ctx.current_fn.unwrap();
    let dispatch_bb = ctx.ictx.append_basic_block(current_fn, "restore.dispatch");
    let norestore_bb = ctx.ictx.append_basic_block(current_fn, "main");
    let cond = gen_compare_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_RESTORE)
        .expect("fail to gen_compare_migration_state");
    ctx.builder
        .build_conditional_branch(cond.into_int_value(), dispatch_bb, norestore_bb)
        .expect("should build conditional branch");

    // dispatch_bb is generated in gen_finalize_restore_dispatch

    ctx.builder.position_at_end(norestore_bb);

    ctx.restore_dispatch_bb = Some(dispatch_bb);
    Ok(())
}

pub fn gen_finalize_restore_dispatch<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<()> {
    let current_fn = ctx.current_fn.unwrap();
    let unreachable_bb = ctx
        .ictx
        .append_basic_block(current_fn, "dispatch.unreachable");
    ctx.builder.position_at_end(unreachable_bb);
    ctx.builder
        .build_unreachable()
        .expect("should build unreachable");

    ctx.builder
        .position_at_end(ctx.restore_dispatch_bb.unwrap());
    let op_index = ctx
        .builder
        .build_call(
            ctx.fn_get_pc_from_frame.unwrap(),
            &[exec_env_ptr.as_basic_value_enum().into()],
            "op_index",
        )
        .expect("should build call");
    ctx.builder
        .build_switch(
            op_index.as_any_value_enum().into_int_value(),
            unreachable_bb,
            &ctx.restore_dispatch_cases,
        )
        .expect("should build switch");
    Ok(())
}

pub fn gen_restore_wasm_stack<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &mut [(PointerValue<'a>, BasicTypeEnum<'a>)],
    before_restore_bb: &BasicBlock<'a>,
    restore_start_bb: &BasicBlock<'a>,
    restore_end_bb: &BasicBlock<'a>,
    // For restoring before function call, function called
    callee: Option<FunctionValue<'a>>,
) -> Result<(BasicBlock<'a>, Option<Vec<BasicValueEnum<'a>>>)> {
    //   ... (in %original_bb)
    //   br restore_op_6_end
    // restore_op_6: (%restore_start_bb)
    //   ...
    //   br restore_op_6_end
    // restore_op_6_end:
    //   phi...
    // ...

    // Restore a frame (locals)
    ctx.builder.position_at_end(*restore_start_bb);
    let mut restored_locals = Vec::new();
    for (_, ty) in locals.iter() {
        let cs = gen_restore_local(ctx, exec_env_ptr, *ty).expect("should build pop_front_local_T");
        restored_locals.push(cs);
    }
    // Add store nodes
    for i in 0..restored_locals.len() {
        let restored_value = &restored_locals[i].try_as_basic_value().left().unwrap();
        let (local_ptr, _) = &locals[i];
        ctx.builder
            .build_store(*local_ptr, *restored_value)
            .expect("should build store");
    }

    // Store stack values
    ctx.builder.position_at_end(*restore_start_bb);
    let frame = ctx.stack_frames.last().expect("frame empty");
    let stack = frame.stack.clone();
    let mut restored_stack = Vec::new();
    let skip_stack_top = if let Some(callee) = callee {
        callee.get_params().len() - 1
    } else {
        0
    };
    for i in 0..(stack.len() - skip_stack_top) {
        let value = stack[i];
        let cs = gen_restore_stack_value(ctx, exec_env_ptr, value.get_type())
            .expect("should build push_T");
        restored_stack.push(cs);
    }

    // Restore args
    let args = if let Some(callee) = callee {
        // Restore arguments if the frame is empty
        let cond = ctx
            .builder
            .build_call(
                ctx.fn_frame_is_empty.unwrap(),
                &[exec_env_ptr.as_basic_value_enum().into()],
                "",
            )
            .expect("should build call");
        let restore_args_bb = ctx.ictx.append_basic_block(
            ctx.current_fn.unwrap(),
            &format!("restore_op_{}.args", ctx.current_op.unwrap()),
        );
        let restore_args_end_bb = ctx.ictx.append_basic_block(
            ctx.current_fn.unwrap(),
            &format!("restore_op_{}.args_end", ctx.current_op.unwrap()),
        );
        ctx.builder
            .build_conditional_branch(
                cond.as_any_value_enum().into_int_value(),
                restore_args_bb,
                restore_args_end_bb,
            )
            .expect("should build conditional branch");

        ctx.builder.position_at_end(restore_args_bb);
        let mut restored_args = Vec::new();
        for p in callee.get_params().iter().skip(1) {
            let ty = p.get_type();
            let restored =
                gen_restore_stack_value(ctx, exec_env_ptr, ty).expect("should build pop_T");
            restored_args.push(restored);
        }
        ctx.builder
            .build_unconditional_branch(restore_args_end_bb)
            .expect("should build unconditional branch");

        // Add phi nodes for args
        // restore_op_N.args:
        //   ...
        //   br %restore_op_N_end
        ctx.builder.position_at_end(restore_args_end_bb);
        let mut args = Vec::new();
        for (i, arg) in restored_args.iter().enumerate() {
            let arg = arg.try_as_basic_value().left().unwrap();
            let ty = arg.get_type();
            let phi = ctx
                .builder
                .build_phi(ty, &format!("arg_{}", i))
                .expect("should build phi");
            phi.add_incoming(&[(&arg, restore_args_bb)]);
            phi.add_incoming(&[(&ty.const_zero(), *restore_start_bb)]);
            args.push(phi.as_basic_value());
        }

        // call pop_front_frame
        ctx.builder
            .build_call(
                ctx.fn_pop_front_frame.unwrap(),
                &[exec_env_ptr.as_basic_value_enum().into()],
                "",
            )
            .expect("should build call");
        ctx.builder
            .build_unconditional_branch(*restore_end_bb)
            .expect("should build unconditional branch");
        Some(args)
    } else {
        // call pop_front_frame
        ctx.builder
            .build_call(
                ctx.fn_pop_front_frame.unwrap(),
                &[exec_env_ptr.as_basic_value_enum().into()],
                "",
            )
            .expect("should build call");
        ctx.builder
            .build_unconditional_branch(*restore_end_bb)
            .expect("should build unconditional branch");
        None
    };
    let restore_bb = ctx.builder.get_insert_block().unwrap();

    // Add phi nodes for restored stack values
    ctx.builder.position_at_end(*restore_end_bb);
    for i in 0..restored_stack.len() {
        let restored_value = &restored_stack[i].try_as_basic_value().left().unwrap();
        let stack_value = stack[i];

        let ty = stack_value.get_type();
        let phi = ctx
            .builder
            .build_phi(ty, &format!("stack_value_{}", i))
            .expect("should build phi");
        phi.add_incoming(&[(&stack_value, *before_restore_bb)]);
        phi.add_incoming(&[(restored_value, restore_bb)]);
        ctx.stack_frames.last_mut().unwrap().stack[i] = phi.as_basic_value();
    }

    ctx.builder.position_at_end(*restore_end_bb);
    Ok((restore_bb, args))
}

pub fn gen_store_globals<'a>(
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
    ctx: &mut Context<'a, '_>,
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

fn gen_set_migration_state<'a>(
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

pub fn gen_checkpoint_before_call<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    let current_fn = ctx.current_fn.expect("should define current_fn");
    let then_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.then");
    let else_bb = ctx.ictx.append_basic_block(current_fn, "chkpt.else");
    let cond = gen_compare_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_CHECKPOINT_START)
        .expect("fail to gen_compare_migration_state");
    ctx.builder
        .build_conditional_branch(cond.into_int_value(), then_bb, else_bb)
        .expect("should build conditional branch");
    ctx.builder.position_at_end(then_bb);
    gen_set_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_CHECKPOINT_CONTINUE)
        .expect("fail to gen_set_migration_state");
    gen_store_frame(ctx, exec_env_ptr, locals).expect("fail to gen_store_frame");
    gen_store_stack(ctx, exec_env_ptr).expect("fail to gen_store_stack");
    gen_return_default_value(ctx).expect("fail to gen_return_default_value");
    ctx.builder.position_at_end(else_bb);
    Ok(())
}

pub fn gen_checkpoint_after_call<'a>(
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
    let migration_state_ptr = ctx
        .builder
        .build_struct_gep(
            ctx.inkwell_types.i32_type,
            *exec_env_ptr,
            *ctx.exec_env_fields.get("migration_state").unwrap(),
            "migration_state_ptr",
        )
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

fn gen_restore_stack_value<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    ty: BasicTypeEnum<'a>,
) -> Result<CallSiteValue<'a>> {
    let cs = if ty.is_int_type() {
        if ty.into_int_type() == ctx.inkwell_types.i32_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_i32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else if ty.into_int_type() == ctx.inkwell_types.i64_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_i64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else {
            bail!("Unsupported type {:?}", ty);
        }
    } else if ty.is_float_type() {
        if ty.into_float_type() == ctx.inkwell_types.f32_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_f32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else if ty.into_float_type() == ctx.inkwell_types.f64_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_f64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else {
            bail!("Unsupported type {:?}", ty)
        }
    } else {
        bail!("Unsupported type {:?}", ty)
    };
    Ok(cs)
}

fn gen_restore_local<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    ty: BasicTypeEnum<'a>,
) -> Result<CallSiteValue<'a>> {
    let cs = if ty.is_int_type() {
        if ty.into_int_type() == ctx.inkwell_types.i32_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_front_local_i32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else if ty.into_int_type() == ctx.inkwell_types.i64_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_front_local_i64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else {
            bail!("Unsupported type {:?}", ty);
        }
    } else if ty.is_float_type() {
        if ty.into_float_type() == ctx.inkwell_types.f32_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_front_local_f32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else if ty.into_float_type() == ctx.inkwell_types.f64_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_front_local_f64.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else {
            bail!("Unsupported type {:?}", ty)
        }
    } else {
        bail!("Unsupported type {:?}", ty)
    };
    Ok(cs)
}
