use anyhow::bail;
use inkwell::AddressSpace;

use crate::context::Context;

pub fn initialize(ctx: &mut Context<'_, '_>) -> anyhow::Result<()> {
    // Define wanco_main function
    let wanco_main_fn_type = ctx.inkwell_types.void_type.fn_type(&[], false);
    let wanco_main_fn = ctx
        .module
        .add_function("wanco_main", wanco_main_fn_type, None);

    // Add basic blocks
    let wanco_entry_block = ctx.ictx.append_basic_block(wanco_main_fn, "entry");
    let wanco_init_block = ctx.ictx.append_basic_block(wanco_main_fn, "init");
    ctx.wanco_init_block = Some(wanco_init_block);
    let wanco_main_block = ctx.ictx.append_basic_block(wanco_main_fn, "main");
    ctx.wanco_main_block = Some(wanco_main_block);

    // Move position to wanco_main entry
    ctx.builder.position_at_end(wanco_entry_block);
    ctx.builder
        .build_unconditional_branch(wanco_init_block)
        .expect("should build unconditional branch (entry -> init)");

    // Define memory_base as Global
    let memory_base_global = ctx.module.add_global(
        ctx.inkwell_types.i8_ptr_type,
        Some(AddressSpace::default()),
        "memory_base",
    );
    memory_base_global.set_initializer(&ctx.inkwell_types.i8_ptr_type.const_zero());
    ctx.global_memory_base = Some(memory_base_global);

    // TODO: how to grow memory?

    define_aux_functions(ctx)?;

    Ok(())
}

pub fn finalize(ctx: &mut Context<'_, '_>) -> anyhow::Result<()> {
    // Move position to wanco_main init
    ctx.builder.position_at_end(
        ctx.wanco_init_block
            .expect("should move to wanco_init_block"),
    );
    ctx.builder
        .build_unconditional_branch(
            ctx.wanco_main_block
                .expect("should define wanco_main_block"),
        )
        .expect("should build unconditional branch (init -> main)");

    // Move position to wanco_main
    ctx.builder.position_at_end(
        ctx.wanco_main_block
            .expect("should move to wanco_main_block"),
    );
    // Call the start function
    let Some(start_idx) = ctx.start_function_idx else {
        bail!("start function not defined");
    };
    let start_fn = ctx.function_values[start_idx as usize];
    ctx.builder
        .build_call(start_fn, &[], "")
        .expect("should build call");

    ctx.builder.build_return(None).expect("should build return");

    Ok(())
}

fn define_aux_functions(_ctx: &mut Context<'_, '_>) -> anyhow::Result<()> {
    // Define print(linm_offset: i32, len: i32)
    /*
    let print_fn_type = ctx.inkwell_types.void_type.fn_type(
        &[
            ctx.inkwell_types.i32_type.into(),
            ctx.inkwell_types.i32_type.into(),
        ],
        false,
    );
    let print_fn = ctx.module.add_function("print", print_fn_type, None);
        let print_block = ctx.ictx.append_basic_block(print_fn, "entry");
        ctx.builder.position_at_end(print_block);
        let mut params = vec![];
        for idx in 0..print_fn.count_params() {
            let v = print_fn.get_nth_param(idx).expect("fail to get_nth_param");
            let ty = print_fn.get_type().get_param_types()[idx as usize];
            let alloca = ctx
                .builder
                .build_alloca(ty, "param")
                .expect("should build alloca");
            ctx.builder
                .build_store(alloca, v)
                .expect("should build store");
            params.push((alloca, ty));
        }
        let linm_offset = ctx
            .builder
            .build_load(params[0].1, params[0].0, "linm_offset")
            .expect("should build load");
        unsafe {
            ctx.builder
                .build_gep(
                    ctx.inkwell_types.i8_ptr_type,
                    ctx.global_memory_base
                        .expect("should define global_memory_base")
                        .as_pointer_value(),
                    &[linm_offset.into_int_value()],
                    "string_ptr",
                )
                .expect("should build gep");
        }

        // TODO: print string

        ctx.builder.build_return(None).expect("should build return");
    */

    Ok(())
}