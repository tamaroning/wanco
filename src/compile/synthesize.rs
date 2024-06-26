use std::collections::HashMap;

use anyhow::bail;
use inkwell::{types::BasicType, values::BasicValue, AddressSpace};

use crate::context::Context;

pub fn initialize(ctx: &mut Context<'_, '_>) -> anyhow::Result<()> {
    // Define ExecEnv struct
    let mut exec_env_fields = HashMap::new();
    exec_env_fields.insert("memory_base", 0);
    exec_env_fields.insert("memory_size", 1);
    exec_env_fields.insert("checkpoint", 2);
    let exec_env_type = ctx.ictx.struct_type(
        &[
            ctx.inkwell_types.i8_ptr_type.into(),
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

    Ok(())
}

pub fn finalize(ctx: &mut Context<'_, '_>) -> anyhow::Result<()> {
    let aot_main = ctx
        .module
        .get_function("aot_main")
        .expect("should define aot_main");
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

    ctx.builder.build_return(None).expect("should build return");

    Ok(())
}
