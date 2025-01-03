use inkwell::{
    debug_info::{AsDIScope, DIFile, DISubprogram},
    llvm_sys::debuginfo::LLVMDIFlagZero,
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
        "<unknown>",
        true,
        "",
        0,
        "<unknown>",
        inkwell::debug_info::DWARFEmissionKind::LineTablesOnly,
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
    subprogram: &DISubprogram<'a>,
) -> inkwell::debug_info::DILocation<'a> {
    let disubprogram_scope = subprogram.as_debug_info_scope();
    ctx.debug_builder
        .create_debug_location(ctx.ictx, insn_offset, 0, disubprogram_scope, None)
}

pub fn create_subprogram_info<'a, 'b>(
    ctx: &Context<'a, 'b>,
    func_index: u32,
) -> DISubprogram<'a> {
    let difile = ctx.debug_unit.get_file();
    let difile_scope = difile.as_debug_info_scope();
    let fn_name = format!("func_{}", func_index);
    let subprogram_type =
        ctx.debug_builder
            .create_subroutine_type(difile, None, &[], LLVMDIFlagZero);
    ctx.debug_builder.create_function(
        difile_scope,
        &fn_name,
        None,
        difile,
        0,
        subprogram_type,
        false,
        true,
        0,
        LLVMDIFlagZero,
        true,
    )
}
