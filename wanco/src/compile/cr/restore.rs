use anyhow::{bail, Result};
use inkwell::{
    basic_block::BasicBlock,
    types::{BasicTypeEnum, FunctionType},
    values::{AnyValue, BasicValue, BasicValueEnum, CallSiteValue, PointerValue},
};

use crate::context::Context;

use super::{gen_compare_migration_state, MIGRATION_STATE_RESTORE};

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

pub fn gen_restore_point_before_call<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &mut [(PointerValue<'a>, BasicTypeEnum<'a>)],
    before_restore_bb: BasicBlock<'a>,
    fn_called: FunctionType<'a>,
) -> Result<Vec<BasicValueEnum<'a>>> {
    let current_fn = ctx.current_fn.unwrap();
    let op_index = ctx.current_op.unwrap();
    let restore_start_bb = ctx
        .ictx
        .append_basic_block(current_fn, &format!("restore_op_{}", op_index));
    let restore_end_bb = ctx
        .ictx
        .append_basic_block(current_fn, &format!("restore_op_{}_end", op_index));
    ctx.builder
        .build_unconditional_branch(restore_end_bb)
        .expect("should build unconditional branch");

    ctx.restore_dispatch_cases.push((
        ctx.inkwell_types.i32_type.const_int(op_index as u64, false),
        restore_start_bb,
    ));

    ctx.builder.position_at_end(restore_start_bb);
    let (restore_last_bb, restored_args) = gen_restore_wasm_stack(
        ctx,
        exec_env_ptr,
        locals,
        &before_restore_bb,
        &restore_start_bb,
        &restore_end_bb,
        Some(fn_called),
    )
    .expect("fail to gen_restore_wasm_stack");
    let restored_args = restored_args.unwrap();

    ctx.builder.position_at_end(restore_end_bb);

    let mut args = Vec::new();
    for (i, ty) in fn_called.get_param_types().iter().skip(1).enumerate().rev() {
        let phi = ctx.builder.build_phi(*ty, "").expect("should build phi");
        phi.add_incoming(&[(&ctx.pop().expect("stack empty"), before_restore_bb)]);
        phi.add_incoming(&[(&restored_args[i], restore_last_bb)]);
        args.push(phi.as_basic_value());
    }

    Ok(args)
}

pub fn gen_restore_point<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &mut [(PointerValue<'a>, BasicTypeEnum<'a>)],
    current_bb: &BasicBlock<'a>,
) -> Result<BasicBlock<'a>> {
    let current_fn = ctx.current_fn.unwrap();
    let op_index = ctx.current_op.unwrap();
    let restore_start_bb = ctx
        .ictx
        .append_basic_block(current_fn, &format!("restore_op_{}", op_index));
    let restore_end_bb = ctx
        .ictx
        .append_basic_block(current_fn, &format!("restore_op_{}_end", op_index));
    ctx.builder
        .build_unconditional_branch(restore_end_bb)
        .expect("should build unconditional branch");

    ctx.restore_dispatch_cases.push((
        ctx.inkwell_types.i32_type.const_int(op_index as u64, false),
        restore_start_bb,
    ));

    ctx.builder.position_at_end(restore_start_bb);
    let (restore_last_bb, _) = gen_restore_wasm_stack(
        ctx,
        exec_env_ptr,
        locals,
        &current_bb,
        &restore_start_bb,
        &restore_end_bb,
        None,
    )
    .expect("fail to gen_restore_wasm_stack");

    ctx.builder.position_at_end(restore_end_bb);
    Ok(restore_last_bb)
}

// Return the last basic block of the restore process.
// Returns the restored arguments if the callee is provided.
fn gen_restore_wasm_stack<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &mut [(PointerValue<'a>, BasicTypeEnum<'a>)],
    before_restore_bb: &BasicBlock<'a>,
    restore_start_bb: &BasicBlock<'a>,
    restore_end_bb: &BasicBlock<'a>,
    // For restoring before function call, function called
    callee: Option<FunctionType<'a>>,
) -> Result<(BasicBlock<'a>, Option<Vec<BasicValueEnum<'a>>>)> {
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
        callee.get_param_types().len() - 1
    } else {
        0
    };
    for i in 0..(stack.len() - skip_stack_top) {
        let value = stack[i];
        let cs = gen_restore_stack_value(ctx, exec_env_ptr, value.get_type())
            .expect("should build push_T");
        restored_stack.push(cs);
    }

    // call pop_front_frame
    ctx.builder
        .build_call(
            ctx.fn_pop_front_frame.unwrap(),
            &[exec_env_ptr.as_basic_value_enum().into()],
            "",
        )
        .expect("should build call");

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
        for ty in callee.get_param_types().iter().skip(1) {
            let restored =
                gen_restore_stack_value(ctx, exec_env_ptr, *ty).expect("should build pop_T");
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

        ctx.builder
            .build_unconditional_branch(*restore_end_bb)
            .expect("should build unconditional branch");
        Some(args)
    } else {
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
