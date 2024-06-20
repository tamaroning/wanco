use anyhow::Result;
use inkwell::AddressSpace;
use wasmparser::MemorySectionReader;

use crate::context::Context;

pub(super) fn compile_memory_section(
    ctx: &mut Context<'_, '_>,
    memories: MemorySectionReader,
) -> Result<()> {
    // Declare memory size as a global value
    let mut size: u32 = 0;
    for (i, memory) in memories.into_iter().enumerate() {
        let memory = memory?;
        size += memory.initial as u32;
        log::debug!("- memory[{}] = {:?}", i, memory);
    }
    let global = ctx.module.add_global(
        ctx.inkwell_types.i32_type,
        Some(AddressSpace::default()),
        "global_mem_size",
    );
    global.set_initializer(&ctx.inkwell_types.i32_type.const_int(size as u64, false));
    ctx.global_memory_size = Some(global);

    // move position to wanco_main init
    ctx.builder.position_at_end(
        ctx.wanco_init_block
            .expect("should define wasker_init_block"),
    );
    // malloc memory
    /*
    let memory_ptr = ctx
        .builder
        .build_array_malloc(
            ctx.inkwell_types.i8_type,
            ctx.inkwell_types
                .i32_type
                .const_int((size * PAGE_SIZE) as u64, false),
            "memory_ptr",
        )
        .expect("should build malloc");
    ctx.builder
        .build_store::<PointerValue>(
            ctx.global_memory_base
                .expect("should define global_memory_base")
                .as_pointer_value(),
            memory_ptr,
        )
        .expect("should build store");
    */
    Ok(())
}
