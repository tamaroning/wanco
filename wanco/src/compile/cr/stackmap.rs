use crate::{compile::debug, context::Context};
use anyhow::Result;
use inkwell::{
    types::BasicTypeEnum,
    values::{BasicMetadataValueEnum, BasicValue, PointerValue},
};

pub fn get_stackmap_id(ctx: &Context) -> u64 {
    let insn = ctx
        .current_op
        .unwrap_or(debug::FUNCION_START_INSN_OFFSET as u32);
    let func = ctx.current_function_idx.expect("function index not set");
    let id = (func as u64) << 32 | insn as u64;
    id
}

// locals: wasm params and locals
pub fn gen_stackmap<'a>(
    ctx: &mut Context<'a, '_>,
    locals: &[(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    let mut stackmap_args: Vec<BasicMetadataValueEnum<'_>> = vec![];
    // args[0] = stackmap id
    stackmap_args.push(
        ctx.inkwell_types
            .i64_type
            .const_int(get_stackmap_id(ctx), false)
            .into(),
    );
    // args[1] = 0 (padding. We don't use it)
    stackmap_args.push(ctx.inkwell_types.i32_type.const_zero().into());
    // args[2..] = all live variables (locals, params, value stack)
    // populate locals
    for (ptr, _) in locals.iter() {
        stackmap_args.push(ptr.as_basic_value_enum().into());
    }
    // populate value stack
    let stack = ctx.stack_frames.last().expect("stack empty");
    for i in 0..stack.stack.len() {
        let value = stack.stack[i];
        stackmap_args.push(value.into());
    }
    ctx.builder
        .build_call(ctx.inkwell_intrs.experimental_stackmap, &stackmap_args, "")
        .expect("should build call");

    // Embed number of locals into debug info.
    let func = ctx.current_function_idx.unwrap();
    let insn = ctx
        .current_op
        .unwrap_or(debug::FUNCION_START_INSN_OFFSET as u32);
    let local_types = locals
        .iter()
        .map(|(_, ty)| debug::metadata::convert_type_name(ctx, ty))
        .collect::<Vec<_>>();
    let stack_types = stack
        .stack
        .iter()
        .map(|v| debug::metadata::convert_type_name(ctx, &v.get_type()))
        .collect::<Vec<_>>();
    ctx.patchpoint_metavalues
        .push(debug::PatchpointMetadataEntry {
            func: func as u32,
            insn: insn as u32,
            locals: local_types,
            stack: stack_types,
        });

    Ok(())
}
