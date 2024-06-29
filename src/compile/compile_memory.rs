use anyhow::Result;
use wasmparser::MemorySectionReader;

use crate::context::Context;

pub(super) fn compile_memory_section(
    ctx: &mut Context<'_, '_>,
    memories: MemorySectionReader,
) -> Result<()> {
    // Set initial memory size
    let mut size: u32 = 0;
    for (i, memory) in memories.into_iter().enumerate() {
        let memory = memory?;
        size += memory.initial as u32;
        log::debug!("- memory[{}] = {:?}", i, memory);
    }
    let init_memory_size =
        ctx.module
            .add_global(ctx.inkwell_types.i32_type, None, "INIT_MEMORY_SIZE");
    init_memory_size.set_initializer(&ctx.inkwell_types.i32_type.const_int(size as u64, false));
    init_memory_size.set_constant(true);

    // move position to aot_main %init
    ctx.builder
        .position_at_end(ctx.wanco_init_block.expect("should define aot_main %init"));
    Ok(())
}
