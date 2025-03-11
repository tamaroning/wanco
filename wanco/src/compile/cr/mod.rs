use anyhow::Result;
use checkpoint::gen_checkpoint_start;
use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValue, BasicValueEnum, PointerValue},
};
use restore::gen_restore_point;

use crate::context::Context;

pub(crate) mod checkpoint;
pub(crate) mod restore;

pub(crate) const MIGRATION_STATE_NONE: i32 = 0;
pub(crate) const MIGRATION_STATE_CHECKPOINT_START: i32 = 1;
pub(crate) const MIGRATION_STATE_CHECKPOINT_CONTINUE: i32 = 2;
pub(crate) const MIGRATION_STATE_RESTORE: i32 = 3;

pub(crate) const MAX_LOCALS_STORE: usize = 10000;
pub(crate) const MAX_STACK_STORE: usize = 10000;

fn gen_migration_state<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
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
    let migration_state = ctx
        .builder
        .build_load(
            ctx.inkwell_types.i32_type,
            migration_state_ptr,
            "migration_state",
        )
        .expect("fail to build load");
    let load = migration_state.as_instruction_value().unwrap();
    load.set_volatile(true).expect("fail to set_volatile");
    Ok(migration_state)
}

pub(super) fn gen_compare_migration_state<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    migration_state: i32,
) -> Result<BasicValueEnum<'a>> {
    let current_migration_state =
        gen_migration_state(ctx, exec_env_ptr).expect("fail to gen_migration_state");

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

pub(self) fn gen_set_migration_state<'a>(
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

// almost equiavalent call both to gen_checkpoint and gen_restore, but emit more efficient code
// by wrapping them in a single conditional branch
// TODO: 最後のブロックと、ローカル変数、スタックを返す
pub(crate) fn gen_migration_point<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    let chkpt_bb = ctx.ictx.append_basic_block(
        ctx.current_fn.unwrap(),
        &format!("chkpt_op_{}.start", ctx.current_op.unwrap()),
    );
    let chkpt_else_bb = ctx.ictx.append_basic_block(
        ctx.current_fn.unwrap(),
        &format!("chkpt_op_{}.else", ctx.current_op.unwrap()),
    );

    let current_migration_state =
        gen_migration_state(ctx, exec_env_ptr).expect("fail to gen_migration_state");
    ctx.builder
        .build_switch(
            current_migration_state.into_int_value(),
            chkpt_else_bb,
            &[(
                ctx.inkwell_types
                    .i32_type
                    .const_int(MIGRATION_STATE_CHECKPOINT_START as u64, false),
                chkpt_bb,
            )],
        )
        .expect("fail to build_switch");

    // checkpoint
    ctx.builder.position_at_end(chkpt_bb);
    
    // start unwinding
    gen_checkpoint_start(ctx, exec_env_ptr, locals).expect("fail to gen_checkpoint");

    // restore (create new bb)
    let phi_bb = ctx.ictx.append_basic_block(
        ctx.current_fn.unwrap(),
        &format!("restore_op_{}.end", ctx.current_op.unwrap()),
    );
    ctx.builder.position_at_end(chkpt_else_bb);
    ctx.builder.build_unconditional_branch(phi_bb).unwrap();
    gen_restore_point(
        ctx,
        exec_env_ptr,
        locals,
        0,
        &phi_bb,
        &ctx.builder.get_insert_block().unwrap(),
    );

    ctx.builder.position_at_end(phi_bb);
    Ok(())
}

pub(crate) fn gen_restore_non_leaf<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
    skip_stack_top: usize,
) -> Result<()> {
    let original_bb = ctx.builder.get_insert_block().unwrap();
    let phi_bb = ctx.ictx.append_basic_block(
        ctx.current_fn.unwrap(),
        &format!("non_leaf_op_{}_restore.end", ctx.current_op.unwrap()),
    );
    ctx.builder.build_unconditional_branch(phi_bb).unwrap();

    gen_restore_point(
        ctx,
        exec_env_ptr,
        locals,
        skip_stack_top,
        &phi_bb,
        &original_bb,
    );
    ctx.builder.position_at_end(phi_bb);
    Ok(())
}
