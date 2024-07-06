use std::collections::HashMap;

use anyhow::bail;
use inkwell::{module::Linkage, types::BasicType, values::BasicValue, AddressSpace};

use crate::context::Context;

use super::checkpoint::gen_store_globals;

pub fn initialize(ctx: &mut Context<'_, '_>) -> anyhow::Result<()> {
    // Define ExecEnv struct
    let mut exec_env_fields = HashMap::new();
    exec_env_fields.insert("memory_base", 0);
    exec_env_fields.insert("memory_size", 1);
    exec_env_fields.insert("migration_state", 2);
    exec_env_fields.insert("private1", 3);
    let exec_env_type = ctx.ictx.struct_type(
        &[
            ctx.inkwell_types.i8_ptr_type.into(),
            ctx.inkwell_types.i32_type.into(),
            ctx.inkwell_types.i32_type.into(),
            // Reserve 4 bytes for checkpoint
            ctx.inkwell_types.i8_ptr_type.into(),
        ],
        false,
    );
    ctx.exec_env_type = Some(exec_env_type);
    ctx.exec_env_fields = exec_env_fields;

    // Define aot_main function
    let aot_main_fn_type = ctx.inkwell_types.void_type.fn_type(
        &[exec_env_type
            .ptr_type(AddressSpace::default())
            .as_basic_type_enum()
            .into()],
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
    // Checkpoint related
    if ctx.config.checkpoint {
        let exec_env_ptr_type = ctx.exec_env_type.unwrap().ptr_type(AddressSpace::default());
        let fn_type_new_frame = ctx
            .inkwell_types
            .void_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_new_frame = Some(ctx.module.add_function(
            "new_frame",
            fn_type_new_frame,
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
        let fn_type_add_local_i32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i32_type.into()],
            false,
        );
        ctx.fn_add_local_i32 = Some(ctx.module.add_function(
            "add_local_i32",
            fn_type_add_local_i32,
            Some(Linkage::External),
        ));
        let fn_type_add_local_i64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i64_type.into()],
            false,
        );
        ctx.fn_add_local_i64 = Some(ctx.module.add_function(
            "add_local_i64",
            fn_type_add_local_i64,
            Some(Linkage::External),
        ));
        let fn_type_add_local_f32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f32_type.into()],
            false,
        );
        ctx.fn_add_local_f32 = Some(ctx.module.add_function(
            "add_local_f32",
            fn_type_add_local_f32,
            Some(Linkage::External),
        ));
        let fn_type_add_local_f64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f64_type.into()],
            false,
        );
        ctx.fn_add_local_f64 = Some(ctx.module.add_function(
            "add_local_f64",
            fn_type_add_local_f64,
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
        let fn_type_add_global_i32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i32_type.into()],
            false,
        );
        ctx.fn_add_global_i32 = Some(ctx.module.add_function(
            "add_global_i32",
            fn_type_add_global_i32,
            Some(Linkage::External),
        ));
        let fn_type_add_global_i64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i64_type.into()],
            false,
        );
        ctx.fn_add_global_i64 = Some(ctx.module.add_function(
            "add_global_i64",
            fn_type_add_global_i64,
            Some(Linkage::External),
        ));
        let fn_type_add_global_f32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f32_type.into()],
            false,
        );
        ctx.fn_add_global_f32 = Some(ctx.module.add_function(
            "add_global_f32",
            fn_type_add_global_f32,
            Some(Linkage::External),
        ));
        let fn_type_add_global_f64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f64_type.into()],
            false,
        );
        ctx.fn_add_global_f64 = Some(ctx.module.add_function(
            "add_global_f64",
            fn_type_add_global_f64,
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

    // Call the start WASM function
    let Some(start_idx) = ctx.start_function_idx else {
        bail!("start function not defined");
    };
    let start_fn = ctx.function_values[start_idx as usize];
    ctx.builder
        .build_call(start_fn, &[exec_env_ptr.as_basic_value_enum().into()], "")
        .expect("should build call");

    if ctx.config.checkpoint {
        // store globals
        gen_store_globals(ctx, &exec_env_ptr).expect("should gen store globals");
    }

    ctx.builder.build_return(None).expect("should build return");

    Ok(())
}
