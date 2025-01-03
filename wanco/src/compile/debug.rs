use inkwell::{
    debug_info::{AsDIScope, DILexicalBlock, DISubprogram},
    llvm_sys::debuginfo::{LLVMDIFlagPublic, LLVMDIFlagZero},
};

use crate::context::Context;

pub fn create_debug_info_builder<'a>(
    module: &inkwell::module::Module<'a>,
) -> (
    inkwell::debug_info::DebugInfoBuilder<'a>,
    inkwell::debug_info::DICompileUnit<'a>,
) {
    module.create_debug_info_builder(
        true,
        inkwell::debug_info::DWARFSourceLanguage::C,
        "<unknown>",
        "<unknown>",
        "wanco",
        true,
        "",
        0,
        "",
        inkwell::debug_info::DWARFEmissionKind::Full,
        0,
        false,
        false,
        "<unknown>",
        "<unknown>",
    )
}

pub fn create_source_location<'a, 'b>(
    ctx: &Context<'a, 'b>,
    func_index: u32,
    insn_offset: u32,
    function_lexical_scope: &DILexicalBlock<'a>,
) -> inkwell::debug_info::DILocation<'a> {
    // We just use function indices as line numbers and instruction offsets as column numbers.
    ctx.debug_builder.create_debug_location(
        ctx.ictx,
        func_index,
        insn_offset,
        function_lexical_scope.as_debug_info_scope(),
        None,
    )
}

pub fn create_subprogram_info<'a, 'b>(ctx: &Context<'a, 'b>, func_index: u32) -> DISubprogram<'a> {
    let file = ctx.debug_unit.get_file();
    let scope = ctx.debug_unit.as_debug_info_scope();
    let fn_name = format!("func_{}", func_index);
    // Use the function type () -> () for now.
    let subprogram_type = ctx
        .debug_builder
        .create_subroutine_type(file, None, &[], LLVMDIFlagZero);
    ctx.debug_builder.create_function(
        scope,
        &fn_name,
        Some(&fn_name),
        file,
        func_index,
        subprogram_type,
        true,
        true,
        func_index,
        LLVMDIFlagPublic,
        true,
    )
}

pub fn create_function_lexical_scope<'a, 'b>(
    ctx: &Context<'a, 'b>,
    function_index: u32,
    subprogram: &DISubprogram<'a>,
) -> DILexicalBlock<'a> {
    ctx.debug_builder.create_lexical_block(
        subprogram.as_debug_info_scope(),
        ctx.debug_unit.get_file(),
        function_index,
        0,
    )
}
