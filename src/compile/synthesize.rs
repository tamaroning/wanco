use std::collections::HashMap;

use anyhow::bail;
use inkwell::{
    attributes::{Attribute, AttributeLoc},
    module::Linkage,
    types::BasicType,
    values::BasicValue,
    AddressSpace,
};

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

    if ctx.config.unwind {
        // new_frame
        let exec_env_ptr_type = ctx.exec_env_type.unwrap().ptr_type(AddressSpace::default());
        let f = ctx.module.add_function(
            "throw_exception",
            ctx.inkwell_types.void_type.fn_type(&[], false),
            Some(Linkage::External),
        );
        let attr_noreturn = ctx
            .ictx
            .create_enum_attribute(Attribute::get_named_enum_kind_id("noreturn"), 0);
        f.add_attribute(AttributeLoc::Function, attr_noreturn);
        ctx.fn_throw_exception = Some(f);
        let fn_type_new_frame = ctx
            .inkwell_types
            .void_type
            .fn_type(&[exec_env_ptr_type.into()], false);
        ctx.fn_new_frame = Some(ctx.module.add_function(
            "new_frame",
            fn_type_new_frame,
            Some(Linkage::External),
        ));
        // add_local_i32
        let fn_type_add_local_i32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i32_type.into()],
            false,
        );
        ctx.fn_add_local_i32 = Some(ctx.module.add_function(
            "add_local_i32",
            fn_type_add_local_i32,
            Some(Linkage::External),
        ));
        // add_local_i64
        let fn_type_add_local_i64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.i64_type.into()],
            false,
        );
        ctx.fn_add_local_i64 = Some(ctx.module.add_function(
            "add_local_i64",
            fn_type_add_local_i64,
            Some(Linkage::External),
        ));
        // add_local_f32
        let fn_type_add_local_f32 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f32_type.into()],
            false,
        );
        ctx.fn_add_local_f32 = Some(ctx.module.add_function(
            "add_local_f32",
            fn_type_add_local_f32,
            Some(Linkage::External),
        ));
        // add_local_f64
        let fn_type_add_local_f64 = ctx.inkwell_types.void_type.fn_type(
            &[exec_env_ptr_type.into(), ctx.inkwell_types.f64_type.into()],
            false,
        );
        ctx.fn_add_local_f64 = Some(ctx.module.add_function(
            "add_local_f64",
            fn_type_add_local_f64,
            Some(Linkage::External),
        ));
    }

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
    if ctx.config.unwind {
        let then_block = ctx.ictx.append_basic_block(aot_main, "invoke.then");
        let catch_block = ctx.ictx.append_basic_block(aot_main, "invoke.catch");
        ctx.builder
            .build_invoke(
                start_fn,
                &[exec_env_ptr.as_basic_value_enum().into()],
                then_block,
                catch_block,
                "",
            )
            .expect("should build invoke");
        ctx.builder.position_at_end(catch_block);
        let null = ctx.inkwell_types.i8_ptr_type.const_null();
        let res = ctx
            .builder
            .build_landing_pad(
                ctx.exception_type,
                ctx.personality_function,
                &[null.into()],
                false,
                "",
            )
            .expect("should build landing pad");
        // TODO: store globals
        ctx.builder.build_resume(res).expect("should build resume");

        ctx.builder.position_at_end(then_block);
    } else {
        ctx.builder
            .build_call(start_fn, &[exec_env_ptr.as_basic_value_enum().into()], "")
            .expect("should build call");
    }

    ctx.builder.build_return(None).expect("should build return");

    Ok(())
}
