use anyhow::{bail, Result};
use inkwell::{
    types::BasicTypeEnum,
    values::{BasicValue, BasicValueEnum},
    AddressSpace,
};
use wasmparser::{DataKind, DataSectionReader, GlobalSectionReader, Operator};

use crate::{
    compile::{compile_type::wasmty_to_llvmty, helper::gen_memory_base},
    context::{Context, Global},
};

pub(super) fn compile_global_section(
    ctx: &mut Context<'_, '_>,
    globals: GlobalSectionReader,
) -> Result<()> {
    // Hold function signature
    // These functions will be registerd in ExportSection
    for (i, global) in globals.into_iter().enumerate() {
        let global = global?;
        let gname = format!("global_{}", i);
        let ty = wasmty_to_llvmty(ctx, &global.ty.content_type)?;

        // Get initial value
        let init_expr_binary_reader = &mut global.init_expr.get_binary_reader();
        let init_val: BasicValueEnum = match init_expr_binary_reader
            .read_operator()
            .expect("fail read_operator")
        {
            Operator::I32Const { value } => ty
                .into_int_type()
                .const_int(value as u64, false)
                .as_basic_value_enum(),
            Operator::I64Const { value } => ty
                .into_int_type()
                .const_int(value as u64, false)
                .as_basic_value_enum(),
            Operator::F32Const { value } => ty
                .into_float_type()
                .const_float(f32::from_bits(value.bits()).into())
                .as_basic_value_enum(),
            Operator::F64Const { value } => ty
                .into_float_type()
                .const_float(f64::from_bits(value.bits()))
                .as_basic_value_enum(),
            _ => {
                bail!("Unsupposed Global const value");
            }
        };

        // declare
        if global.ty.mutable {
            // Declare GlobalValue
            let global_value = ctx
                .module
                .add_global(ty, Some(AddressSpace::default()), &gname);
            match init_val.get_type() {
                BasicTypeEnum::IntType(..) => {
                    global_value.set_initializer(&init_val.into_int_value());
                }
                BasicTypeEnum::FloatType(..) => {
                    global_value.set_initializer(&init_val.into_float_value());
                }
                _ => {
                    bail!("Unsupposed Global mutable value");
                }
            }
            ctx.globals.push(Global::Mut {
                ptr: global_value,
                ty,
            });
            global_value.set_initializer(&init_val);
        } else {
            // declare as BasicValueEnum
            ctx.globals.push(Global::Const { value: init_val });
        }
    }
    log::debug!("- declare {} globals", ctx.globals.len());
    Ok(())
}

pub(super) fn compile_data_section<'a>(
    ctx: &mut Context<'a, '_>,
    data_segs: DataSectionReader,
) -> Result<()> {
    // Move position to aot_main %init
    ctx.builder
        .position_at_end(ctx.aot_init_block.expect("should define aot_main %init"));

    for data in data_segs {
        let data = data?;
        log::debug!(
            "DataSection kind:{:?},  range:{}-{}",
            data.kind,
            data.range.start,
            data.range.end
        );
        match data.kind {
            DataKind::Passive => {
                log::error!("DataKind::Passive is not supported")
            }
            DataKind::Active {
                memory_index: _,
                offset_expr,
            } => {
                // Make array from data
                let size = data.data.len();
                log::debug!("- data size = {}", size);
                let array_ty = ctx.inkwell_types.i8_type.array_type(size as u32);
                let data_segment_initializer = ctx.module.add_global(
                    array_ty,
                    Some(AddressSpace::default()),
                    "global_memory_initializer",
                );

                // Initialize array
                let mut data_intvalue = Vec::new();
                for d in data.data {
                    let d_intvalue = ctx.inkwell_types.i8_type.const_int(*d as u64, false);
                    data_intvalue.push(d_intvalue);
                }
                log::debug!("- data_intvalue.len = {}", data_intvalue.len());
                let initializer = ctx.inkwell_types.i8_type.const_array(&data_intvalue);
                data_segment_initializer.set_initializer(&initializer);

                // Get offset from the base of the Linear Memory
                let offset_op = offset_expr
                    .get_binary_reader()
                    .read_operator()
                    .expect("failed to get data section offset");
                let offset = match offset_op {
                    Operator::I32Const { value } => value,
                    _ => unreachable!("unsupported offset type"),
                };
                log::debug!("- offset = 0x{:x}", offset);
                let offset_int = ctx.inkwell_types.i64_type.const_int(offset as u64, false);

                // move position to aot_main init
                ctx.builder
                    .position_at_end(ctx.aot_init_block.expect("should define aot_init_block"));
                let exec_env_ptr = ctx
                    .module
                    .get_function("aot_main")
                    .expect("should define aot_main")
                    .get_first_param()
                    .expect("should have &exec_env")
                    .into_pointer_value();
                let memory_base =
                    gen_memory_base(ctx, &exec_env_ptr).expect("should gen memory_base");
                let memory_base_int = ctx
                    .builder
                    .build_ptr_to_int(memory_base, ctx.inkwell_types.i64_type, "memory_base_int")
                    .expect("should build ptr to int");
                let dest_int =
                    ctx.builder
                        .build_int_add(memory_base_int, offset_int, "dest_int")?;
                let dest_ptr = ctx
                    .builder
                    .build_int_to_ptr(dest_int, ctx.inkwell_types.i8_ptr_type, "dest_ptr")
                    .expect("should build int to ptr");

                // Memcpy from initializer to Linear Memory
                ctx.builder
                    .build_memcpy(
                        dest_ptr,
                        1,
                        data_segment_initializer.as_pointer_value(),
                        1,
                        ctx.inkwell_types
                            .i64_type
                            .const_int(data.data.len() as u64, false),
                    )
                    .expect("should build memcpy");
            }
        }
    }
    Ok(())
}
