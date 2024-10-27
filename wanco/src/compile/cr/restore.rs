use anyhow::{bail, Result};
use inkwell::{
    basic_block::BasicBlock,
    types::{BasicTypeEnum, FunctionType},
    values::{AnyValue, BasicValue, BasicValueEnum, CallSiteValue, PointerValue},
};

use crate::context::{Context, Global};

use super::{gen_compare_migration_state, MIGRATION_STATE_RESTORE};

pub(crate) fn gen_restore_dispatch<'a>(
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

pub(crate) fn gen_finalize_restore_dispatch<'a>(
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

// post: phiにphiノードを追加して、builderのカーソルを移動する
// original_bb: phiで合流元のブロック
// phi_bb: phiで合流先のブロック
pub(crate) fn gen_restore_point<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
    skip_stack_top: usize,
    phi_bb: &BasicBlock<'a>,
    original_bb: &BasicBlock<'a>,
) {
    let current_fn = ctx.current_fn.unwrap();
    let op_index = ctx.current_op.unwrap();
    let restore_start_bb = ctx
        .ictx
        .append_basic_block(current_fn, &format!("restore_op_{}.start", op_index));

    ctx.restore_dispatch_cases.push((
        ctx.inkwell_types.i32_type.const_int(op_index as u64, false),
        restore_start_bb,
    ));

    ctx.builder.position_at_end(restore_start_bb);
    gen_restore_wasm_stack(
        ctx,
        exec_env_ptr,
        locals,
        skip_stack_top,
        &restore_start_bb,
        &phi_bb,
        &original_bb,
    );
}

// Return the last basic block of the restore process.
// Returns the restored arguments if the callee is provided.
// post: phi_bbにphiノードを追加して、builderのカーソルを移動する
fn gen_restore_wasm_stack<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
    skip_stack_top: usize,
    restore_start_bb: &BasicBlock<'a>,
    phi_bb: &BasicBlock<'a>,
    original_bb: &BasicBlock<'a>,
) {
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
    for i in 0..stack.len() {
        let value_type = stack[i].get_type();
        if stack.len() - i <= skip_stack_top {
            // argumentなどのスキップするスタックトップの値は0で埋める
            // (unwind時には、pushされないので)
            restored_stack.push(value_type.const_zero());
        } else {
            let cs = gen_restore_stack_value(ctx, exec_env_ptr, value_type)
                .expect("should build push_T")
                .try_as_basic_value()
                .left()
                .unwrap();
            restored_stack.push(cs);
        }
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
        .build_unconditional_branch(*phi_bb)
        .expect("should build unconditional branch");

    // Add phi nodes for restored stack values
    ctx.builder.position_at_end(*phi_bb);
    for i in 0..restored_stack.len() {
        let restored_value = &restored_stack[i];
        let stack_value = stack[i];

        let ty = stack_value.get_type();
        let phi = ctx
            .builder
            .build_phi(ty, &format!("stack_value_{}", i))
            .expect("should build phi");
        phi.add_incoming(&[(&stack_value, *original_bb)]);
        phi.add_incoming(&[(restored_value, *restore_start_bb)]);
        ctx.stack_frames.last_mut().unwrap().stack[i] = phi.as_basic_value();
    }
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

pub(crate) fn gen_restore_globals<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<()> {
    let current_fn = ctx.current_fn.unwrap();
    let then_bb = ctx.ictx.append_basic_block(current_fn, "restore.then");
    let else_bb = ctx.ictx.append_basic_block(current_fn, "restore.else");
    let cond = gen_compare_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_RESTORE)
        .expect("fail to gen_compare_migration_state");
    ctx.builder
        .build_conditional_branch(cond.into_int_value(), then_bb, else_bb)
        .expect("should build conditional branch");
    ctx.builder.position_at_end(then_bb);

    // add globals
    for global in &ctx.globals {
        match global {
            Global::Const { value } => {
                let ty = value.get_type();
                gen_restore_global(ctx, exec_env_ptr, ty)
                    .expect("should build pop_front_global for mut global");
            }
            Global::Mut { ptr, ty } => {
                let v = gen_restore_global(ctx, exec_env_ptr, *ty)
                    .expect("should build pop_front_global for mut global");
                let v = v.try_as_basic_value().left().unwrap();
                ctx.builder
                    .build_store(ptr.as_pointer_value(), v)
                    .expect("should build load");
            }
        };
    }
    ctx.builder
        .build_unconditional_branch(else_bb)
        .expect("should build unconditonal branch");
    // Move back to else bb
    ctx.builder.position_at_end(else_bb);
    Ok(())
}

fn gen_restore_global<'a>(
    ctx: &Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    ty: BasicTypeEnum<'a>,
) -> Result<CallSiteValue<'a>> {
    let cs = if ty.is_int_type() {
        if ty.into_int_type() == ctx.inkwell_types.i32_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_front_global_i32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else if ty.into_int_type() == ctx.inkwell_types.i64_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_front_global_i64.unwrap(),
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
                    ctx.fn_pop_front_global_f32.unwrap(),
                    &[exec_env_ptr.as_basic_value_enum().into()],
                    "",
                )
                .expect("should build call")
        } else if ty.into_float_type() == ctx.inkwell_types.f64_type {
            ctx.builder
                .build_call(
                    ctx.fn_pop_front_global_f64.unwrap(),
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

pub(crate) fn gen_restore_table<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<()> {
    let Some(global_table) = ctx.global_table else {
        return Ok(());
    };
    let current_fn = ctx.current_fn.unwrap();
    let then_bb = ctx.ictx.append_basic_block(current_fn, "restore.then");
    let else_bb = ctx.ictx.append_basic_block(current_fn, "restore.else");
    let cond = gen_compare_migration_state(ctx, exec_env_ptr, MIGRATION_STATE_RESTORE)
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
        let value = ctx
            .builder
            .build_call(
                ctx.fn_pop_front_table_index.unwrap(),
                &[exec_env_ptr.as_basic_value_enum().into()],
                "",
            )
            .expect("should build call");
        let value = value.as_any_value_enum().into_int_value();
        ctx.builder
            .build_store(elem_ptr, value)
            .expect("should build store");
    }

    ctx.builder
        .build_unconditional_branch(else_bb)
        .expect("should build unconditonal branch");
    ctx.builder.position_at_end(else_bb);
    Ok(())
}
