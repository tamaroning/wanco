use std::collections::HashMap;

use anyhow::bail;
use inkwell::{module::Linkage, types::BasicType, values::BasicValue};

use crate::context::Context;

use super::cr::{
    checkpoint::{add_fn_store_globals, add_fn_store_table, gen_store_globals_and_table},
    restore::{gen_restore_globals, gen_restore_table},
};

pub fn initialize(ctx: &mut Context<'_, '_>) -> anyhow::Result<()> {
    // Define ExecEnv struct
    // See lib-rt/aot.h for the type definition
    let mut exec_env_fields = HashMap::new();
    exec_env_fields.insert("memory_base", 0);
    exec_env_fields.insert("memory_size", 1);
    exec_env_fields.insert("migration_state", 2);
    exec_env_fields.insert("argc", 3);
    exec_env_fields.insert("argv", 4);
    let exec_env_type = ctx.ictx.struct_type(
        &[
            ctx.inkwell_types.ptr_type.into(),
            ctx.inkwell_types.i32_type.into(),
            ctx.inkwell_types.i32_type.into(),
            ctx.inkwell_types.i32_type.into(),
            ctx.inkwell_types.ptr_type.into(),
        ],
        false,
    );
    ctx.exec_env_type = Some(exec_env_type);
    ctx.exec_env_fields = exec_env_fields;

    // Define aot_main function
    let aot_main_fn_type = ctx.inkwell_types.void_type.fn_type(
        &[ctx.inkwell_types.ptr_type.as_basic_type_enum().into()],
        false,
    );
    let aot_main_fn = ctx.module.add_function("aot_main", aot_main_fn_type, None);

    // Add basic blocks
    let aot_entry_block = ctx.ictx.append_basic_block(aot_main_fn, "entry");
    let aot_init_block = ctx.ictx.append_basic_block(aot_main_fn, "init");
    ctx.aot_init_block = Some(aot_init_block);
    let aot_main_block = ctx.ictx.append_basic_block(aot_main_fn, "main");
    ctx.aot_main_block = Some(aot_main_block);

    // Move position to aot_main %entry
    ctx.builder.position_at_end(aot_entry_block);
    // br %init
    ctx.builder
        .build_unconditional_branch(aot_init_block)
        .expect("should build unconditional branch (entry -> init)");

    // Declare memory_grow function
    let fn_type_memory_grow = ctx
        .inkwell_types
        .i32_type
        .fn_type(&[ctx.inkwell_types.i32_type.into()], false);
    let fn_memory_grow = ctx
        .module
        .add_function("memory_grow", fn_type_memory_grow, None);
    ctx.fn_memory_grow = Some(fn_memory_grow);

    load_api(ctx);
    Ok(())
}

pub fn load_api(ctx: &mut Context<'_, '_>) {
    let exec_env_ptr_type = ctx.inkwell_types.ptr_type;
    // Checkpoint related
    // FIXME: We should only add these functions if we are using checkpointing
    // However, lib-rt statically links fn_store_globals and fn_store_table
    if true || ctx.config.enable_cr || ctx.config.legacy_cr {
        // checkpoint api
        let fn_type_start_checkpoint = ctx
            .inkwell_types
            .void_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_start_checkpoint = Some(ctx.module.add_function(
            "start_checkpoint",
            fn_type_start_checkpoint,
            Some(Linkage::External),
        ));

        let fn_type_push_frame = ctx
            .inkwell_types
            .void_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_push_frame = Some(ctx.module.add_function(
            "push_frame",
            fn_type_push_frame,
            Some(Linkage::External),
        ));
        let fn_type_set_pc_to_frame = ctx.inkwell_types.void_type.fn_type(
            &[
                exec_env_ptr_type.into(),
                ctx.inkwell_types.i32_type.into(),
                ctx.inkwell_types.i32_type.into(),
            ],
            false,
        );
        ctx.fn_set_pc_to_frame = Some(ctx.module.add_function(
            "set_pc_to_frame",
            fn_type_set_pc_to_frame,
            Some(Linkage::External),
        ));
        let fn_type_push_local_i32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i32_type.into()],
            false,
        );
        ctx.fn_push_local_i32 = Some(ctx.module.add_function(
            "push_local_i32",
            fn_type_push_local_i32,
            Some(Linkage::External),
        ));
        let fn_type_push_local_i64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i64_type.into()],
            false,
        );
        ctx.fn_push_local_i64 = Some(ctx.module.add_function(
            "push_local_i64",
            fn_type_push_local_i64,
            Some(Linkage::External),
        ));
        let fn_type_push_local_f32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f32_type.into()],
            false,
        );
        ctx.fn_push_local_f32 = Some(ctx.module.add_function(
            "push_local_f32",
            fn_type_push_local_f32,
            Some(Linkage::External),
        ));
        let fn_type_push_local_f64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f64_type.into()],
            false,
        );
        ctx.fn_push_local_f64 = Some(ctx.module.add_function(
            "push_local_f64",
            fn_type_push_local_f64,
            Some(Linkage::External),
        ));
        let fn_type_push_i32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i32_type.into()],
            false,
        );
        ctx.fn_push_i32 = Some(ctx.module.add_function(
            "push_i32",
            fn_type_push_i32,
            Some(Linkage::External),
        ));
        let fn_type_push_i64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i64_type.into()],
            false,
        );
        ctx.fn_push_i64 = Some(ctx.module.add_function(
            "push_i64",
            fn_type_push_i64,
            Some(Linkage::External),
        ));
        let fn_type_push_f32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f32_type.into()],
            false,
        );
        ctx.fn_push_f32 = Some(ctx.module.add_function(
            "push_f32",
            fn_type_push_f32,
            Some(Linkage::External),
        ));
        let fn_type_push_f64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f64_type.into()],
            false,
        );
        ctx.fn_push_f64 = Some(ctx.module.add_function(
            "push_f64",
            fn_type_push_f64,
            Some(Linkage::External),
        ));
        let fn_type_push_global_i32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i32_type.into()],
            false,
        );
        ctx.fn_push_global_i32 = Some(ctx.module.add_function(
            "push_global_i32",
            fn_type_push_global_i32,
            Some(Linkage::External),
        ));
        let fn_type_push_global_i64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i64_type.into()],
            false,
        );
        ctx.fn_push_global_i64 = Some(ctx.module.add_function(
            "push_global_i64",
            fn_type_push_global_i64,
            Some(Linkage::External),
        ));
        let fn_type_push_global_f32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f32_type.into()],
            false,
        );
        ctx.fn_push_global_f32 = Some(ctx.module.add_function(
            "push_global_f32",
            fn_type_push_global_f32,
            Some(Linkage::External),
        ));
        let fn_type_push_global_f64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f64_type.into()],
            false,
        );
        ctx.fn_push_global_f64 = Some(ctx.module.add_function(
            "push_global_f64",
            fn_type_push_global_f64,
            Some(Linkage::External),
        ));
        let fn_type_push_table_index = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i32_type.into()],
            false,
        );
        ctx.fn_push_table_index = Some(ctx.module.add_function(
            "push_table_index",
            fn_type_push_table_index,
            Some(Linkage::External),
        ));

        // restore api
        let fn_type_pop_front_frame = ctx
            .inkwell_types
            .void_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_frame = Some(ctx.module.add_function(
            "pop_front_frame",
            fn_type_pop_front_frame,
            Some(Linkage::External),
        ));
        let fn_type_get_pc_from_frame = ctx
            .inkwell_types
            .i32_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_get_pc_from_frame = Some(ctx.module.add_function(
            "get_pc_from_frame",
            fn_type_get_pc_from_frame,
            Some(Linkage::External),
        ));
        let fn_type_frame_is_empty = ctx
            .inkwell_types
            .bool_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_frame_is_empty = Some(ctx.module.add_function(
            "frame_is_empty",
            fn_type_frame_is_empty,
            Some(Linkage::External),
        ));
        // locals
        let fn_type_pop_front_local_i32 = ctx
            .inkwell_types
            .i32_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_local_i32 = Some(ctx.module.add_function(
            "pop_front_local_i32",
            fn_type_pop_front_local_i32,
            Some(Linkage::External),
        ));
        let fn_type_pop_front_local_i64 = ctx
            .inkwell_types
            .i64_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_local_i64 = Some(ctx.module.add_function(
            "pop_front_local_i64",
            fn_type_pop_front_local_i64,
            Some(Linkage::External),
        ));
        let fn_type_pop_front_local_f32 = ctx
            .inkwell_types
            .f32_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_local_f32 = Some(ctx.module.add_function(
            "pop_front_local_f32",
            fn_type_pop_front_local_f32,
            Some(Linkage::External),
        ));
        let fn_type_pop_front_local_f64 = ctx
            .inkwell_types
            .f64_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_local_f64 = Some(ctx.module.add_function(
            "pop_front_local_f64",
            fn_type_pop_front_local_f64,
            Some(Linkage::External),
        ));
        // stack
        let fn_type_pop_i32 = ctx
            .inkwell_types
            .i32_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_i32 = Some(ctx.module.add_function(
            "pop_i32",
            fn_type_pop_i32,
            Some(Linkage::External),
        ));
        let fn_type_pop_i64 = ctx
            .inkwell_types
            .i64_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_i64 = Some(ctx.module.add_function(
            "pop_i64",
            fn_type_pop_i64,
            Some(Linkage::External),
        ));
        let fn_type_pop_f32 = ctx
            .inkwell_types
            .f32_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_f32 = Some(ctx.module.add_function(
            "pop_f32",
            fn_type_pop_f32,
            Some(Linkage::External),
        ));
        let fn_type_pop_f64 = ctx
            .inkwell_types
            .f64_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_f64 = Some(ctx.module.add_function(
            "pop_f64",
            fn_type_pop_f64,
            Some(Linkage::External),
        ));
        // globals
        let fn_type_pop_front_global_i32 = ctx
            .inkwell_types
            .i32_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_global_i32 = Some(ctx.module.add_function(
            "pop_front_global_i32",
            fn_type_pop_front_global_i32,
            Some(Linkage::External),
        ));
        let fn_type_pop_front_global_i64 = ctx
            .inkwell_types
            .i64_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_global_i64 = Some(ctx.module.add_function(
            "pop_front_global_i64",
            fn_type_pop_front_global_i64,
            Some(Linkage::External),
        ));
        let fn_type_pop_front_global_f32 = ctx
            .inkwell_types
            .f32_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_global_f32 = Some(ctx.module.add_function(
            "pop_front_global_f32",
            fn_type_pop_front_global_f32,
            Some(Linkage::External),
        ));
        let fn_type_pop_front_global_f64 = ctx
            .inkwell_types
            .f64_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_global_f64 = Some(ctx.module.add_function(
            "pop_front_global_f64",
            fn_type_pop_front_global_f64,
            Some(Linkage::External),
        ));
        // table
        let fn_type_pop_front_table_index = ctx
            .inkwell_types
            .i32_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_pop_front_table_index = Some(ctx.module.add_function(
            "pop_front_table_index",
            fn_type_pop_front_table_index,
            Some(Linkage::External),
        ));
    }
}

pub fn finalize(ctx: &mut Context<'_, '_>) -> anyhow::Result<()> {
    let aot_main = ctx
        .module
        .get_function("aot_main")
        .expect("should define aot_main");
    ctx.current_fn = Some(aot_main);
    ctx.current_function_idx = None;

    let exec_env_ptr = aot_main.get_first_param().expect("should have &exec_env");
    let exec_env_ptr = exec_env_ptr.into_pointer_value();

    // Move position to aot_main %init
    ctx.builder
        .position_at_end(ctx.aot_init_block.expect("should move to aot_init_block"));
    // br %main
    ctx.builder
        .build_unconditional_branch(ctx.aot_main_block.expect("should define aot_main_block"))
        .expect("should build unconditional branch (init -> main)");

    // Move position to aot_main %init
    ctx.builder
        .position_at_end(ctx.aot_main_block.expect("should move to aot_main_block"));

    // restore globals
    if !ctx.config.no_restore && (ctx.config.enable_cr || ctx.config.legacy_cr) {
        gen_restore_globals(ctx, &exec_env_ptr).expect("should gen restore globals");
        gen_restore_table(ctx, &exec_env_ptr).expect("should gen restore table");
    }

    // Call the start WASM function
    let Some(start_idx) = ctx.start_function_idx else {
        bail!("start function not defined");
    };
    let start_fn = ctx.function_values[start_idx as usize];
    ctx.builder
        .build_call(start_fn, &[exec_env_ptr.as_basic_value_enum().into()], "")
        .expect("should build call");

    // checkpoint globals (legacy)
    if ctx.config.legacy_cr {
        gen_store_globals_and_table(ctx, &exec_env_ptr)?;
    }

    ctx.builder.build_return(None).expect("should build return");

    // add functions to checkpoint globals and table
    // We always add these functions because lib-rt statically links them
    add_fn_store_globals(ctx, exec_env_ptr)?;
    add_fn_store_table(ctx, exec_env_ptr)?;

    Ok(())
}
