use crate::context::Context;
use inkwell::{
    debug_info::{AsDIScope, DILexicalBlock, DISubprogram},
    llvm_sys::debuginfo::{LLVMDIFlagPublic, LLVMDIFlagZero},
};
use serde::Serialize;

// I have no idea why 0xffff_ffff does not work. Just use u16 value for now.
pub const FUNCION_START_INSN_OFFSET: u16 = 0xffff;

pub mod metadata {
    use inkwell::types::BasicTypeEnum;

    use crate::context::Context;

    const I32_TYPE: &str = "i32";
    const I64_TYPE: &str = "i64";
    const F32_TYPE: &str = "f32";
    const F64_TYPE: &str = "f64";

    pub fn convert_type_name(ctx: &Context, ty: &BasicTypeEnum) -> &'static str {
        if ty.is_int_type() {
            let int_ty = ty.into_int_type();
            if int_ty == ctx.inkwell_types.i32_type {
                I32_TYPE
            } else if int_ty == ctx.inkwell_types.i64_type {
                I64_TYPE
            } else {
                panic!("unsupported int type");
            }
        } else if ty.is_float_type() {
            let float_ty = ty.into_float_type();
            if float_ty == ctx.inkwell_types.f32_type {
                F32_TYPE
            } else if float_ty == ctx.inkwell_types.f64_type {
                F64_TYPE
            } else {
                panic!("unsupported float type");
            }
        } else {
            panic!("unsupported type");
        }
    }
}

#[derive(Serialize)]
pub struct PatchpointMetadataEntry {
    // wasm function index
    pub func: u32,
    // wasm instruction offset from the start of the function
    pub insn: u32,
    // types of local variables
    pub locals: Vec<&'static str>,
    // types of stack values
    pub stack: Vec<&'static str>,
}

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

pub fn create_source_location<'a>(
    ctx: &Context<'a, '_>,
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

pub fn create_subprogram_info<'a>(ctx: &Context<'a, '_>, func_index: u32) -> DISubprogram<'a> {
    let file = ctx.debug_cu.get_file();
    let scope = ctx.debug_cu.as_debug_info_scope();
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

pub fn create_function_lexical_scope<'a>(
    ctx: &Context<'a, '_>,
    function_index: u32,
    subprogram: &DISubprogram<'a>,
) -> DILexicalBlock<'a> {
    ctx.debug_builder.create_lexical_block(
        subprogram.as_debug_info_scope(),
        ctx.debug_cu.get_file(),
        function_index,
        0,
    )
}

/// Create a content of .wanco.metadata section.
pub fn create_patchpoint_metadata_json(ctx: &Context<'_, '_>) -> String {
    serde_json::to_string(&ctx.patchpoint_metavalues).unwrap()
}
