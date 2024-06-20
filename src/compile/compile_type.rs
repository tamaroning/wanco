use anyhow::{bail, Result};
use inkwell::types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType};
use wasmparser::{CompositeType, TypeSectionReader, ValType};

use crate::context::Context;

pub(super) fn compile_type_section(
    ctx: &mut Context<'_, '_>,
    types: TypeSectionReader,
) -> Result<()> {
    for entry in types {
        let subtypes: Vec<_> = entry?.into_types().collect();
        assert_eq!(subtypes.len(), 1);
        let CompositeType::Func(ref func_type) = subtypes[0].composite_type else {
            bail!("TypeSection: Unimplemented composite type: {:?}", subtypes);
        };
        log::debug!("- type: {:?}", func_type);

        let params = func_type.params();
        let returns = func_type.results();

        // Convert wasmparser type to inkwell type
        let mut params_llty: Vec<BasicMetadataTypeEnum> = Vec::new();
        for param in params.iter() {
            let param_llty = wasmty_to_llvmty(ctx, *param)?;
            params_llty.push(param_llty.into());
        }

        let sig: FunctionType = match returns.len() {
            0 => {
                let return_llty = ctx.inkwell_types.void_type;
                return_llty.fn_type(&params_llty, false)
            }
            1 => {
                let return_llty = wasmty_to_llvmty(ctx, returns[0]).expect("convert return type");
                return_llty.fn_type(&params_llty, false)
            }
            _ => {
                bail!("TypeSection: Unimplemented multiple return value");
            }
        };
        ctx.signatures.push(sig);
    }
    Ok(())
}

pub(super) fn wasmty_to_llvmty<'a>(
    ctx: &Context<'a, '_>,
    wasmty: ValType,
) -> Result<BasicTypeEnum<'a>> {
    match wasmty {
        ValType::I32 => Ok(BasicTypeEnum::IntType(ctx.inkwell_types.i32_type)),
        ValType::I64 => Ok(BasicTypeEnum::IntType(ctx.inkwell_types.i64_type)),
        ValType::F32 => Ok(BasicTypeEnum::FloatType(ctx.inkwell_types.f32_type)),
        ValType::F64 => Ok(BasicTypeEnum::FloatType(ctx.inkwell_types.f64_type)),
        _ => bail!("Unimplemented ValType: {:?}", wasmty),
    }
}
