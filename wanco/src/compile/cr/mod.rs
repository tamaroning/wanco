use anyhow::Result;
use inkwell::values::{BasicValue, BasicValueEnum, PointerValue};

use crate::context::Context;

pub(crate) mod checkpoint;
pub(crate) mod restore;

pub(crate) const MIGRATION_STATE_NONE: i32 = 0;
pub(crate) const MIGRATION_STATE_CHECKPOINT_START: i32 = 1;
pub(crate) const MIGRATION_STATE_CHECKPOINT_CONTINUE: i32 = 2;
pub(crate) const MIGRATION_STATE_RESTORE: i32 = 3;

pub(self) fn gen_compare_migration_state<'a>(
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
    // Since the load instruction is moved to outside of the loop, we need to set it as volatile
    let load_inst = current_migration_state.as_instruction_value().unwrap();
    load_inst.set_volatile(true).expect("fail to set_volatile");

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
