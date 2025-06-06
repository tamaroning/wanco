use inkwell::{
    types::{BasicType, BasicTypeEnum},
    values::{AnyValue, BasicValue, IntValue, PhiValue, PointerValue},
};
use wasmparser::{FunctionBody, MemArg, Operator};

use crate::{
    compile::{
        compile_type::wasmty_to_llvmty,
        control::{
            gen_block, gen_br, gen_br_table, gen_brif, gen_call, gen_call_indirect, gen_drop,
            gen_else, gen_end, gen_if, gen_loop, gen_return, gen_select, gen_unreachable,
            ControlFrame, UnreachableReason,
        },
        cr::{
            gen_migration_point,
            restore::{gen_finalize_restore_dispatch, gen_restore_dispatch},
        },
        helper::{self, gen_float_compare, gen_int_compare, gen_llvm_intrinsic},
    },
    context::{Context, Global, StackFrame},
};
use anyhow::{anyhow, bail, Context as _, Result};

use super::helper::{gen_memory_base, gen_memory_size};

pub(super) fn compile_function(ctx: &mut Context<'_, '_>, f: FunctionBody) -> Result<()> {
    log::debug!(
        "Compile function (idx = {})",
        ctx.current_function_idx.unwrap()
    );

    let current_fn = ctx.function_values[ctx.current_function_idx.unwrap() as usize];
    ctx.current_fn = Some(current_fn);
    let entry_bb = ctx.ictx.append_basic_block(current_fn, "entry");
    let ret_bb = ctx.ictx.append_basic_block(current_fn, "ret");
    ctx.stack_frames.push(StackFrame::new());

    ctx.builder.position_at_end(ret_bb);
    let ret = current_fn.get_type().get_return_type();
    let mut end_phis: Vec<PhiValue> = Vec::new();
    if let Some(v) = ret {
        log::trace!("- return type {:?}", v);
        let phi = ctx
            .builder
            .build_phi(v, "return_phi")
            .expect("should build phi");
        end_phis.push(phi);
    }

    ctx.builder.position_at_end(entry_bb);
    ctx.control_frames.push(ControlFrame::Block {
        next: ret_bb,
        end_phis,
        stack_size: ctx.current_frame_size(),
    });

    // Allocate space for &exec_env
    let exec_env_ptr = current_fn
        .get_first_param()
        .expect("should have &exec_env as a first param");
    let exec_env_ptr = exec_env_ptr.into_pointer_value();

    // Register all wasm locals (WASM params and WASM locals)
    let mut locals = vec![];

    // Allocate space for params of wasm function
    // Skip allocating space for the first param (i.e. &exec_env)
    assert!(current_fn.get_first_param().unwrap().is_pointer_value());
    for idx in 1..current_fn.count_params() {
        let v = current_fn
            .get_nth_param(idx)
            .expect("fail to get_nth_param");
        let ty = current_fn.get_type().get_param_types()[idx as usize];
        let alloca = ctx
            .builder
            .build_alloca(ty, "param")
            .expect("should build alloca");
        ctx.builder
            .build_store(alloca, v)
            .expect("should build store");
        locals.push((alloca, ty));
    }

    // Allocate space for wasm locals
    let mut local_reader = f.get_locals_reader()?;
    let num_locals = local_reader.get_count();
    for _ in 0..num_locals {
        let (count, valty) = local_reader.read()?;
        let valty = wasmty_to_llvmty(ctx, &valty)?;
        for _ in 0..count {
            let alloca = ctx
                .builder
                .build_alloca(valty, "local")
                .expect("should build alloca");
            ctx.builder
                .build_store(alloca, valty.const_zero())
                .expect("should build store");
            locals.push((alloca, valty));
        }
    }

    // entry dispatcher for restore
    if !ctx.config.no_restore && (ctx.config.enable_cr || ctx.config.legacy_cr) {
        ctx.restore_dispatch_bb = None;
        ctx.restore_dispatch_cases = vec![];
        gen_restore_dispatch(ctx, &exec_env_ptr).expect("should gen restore dispatch")
    }

    // Generate checkpoint
    if ctx.config.enable_cr || ctx.config.legacy_cr {
        ctx.current_op = Some(u32::MAX);
        gen_migration_point(ctx, &exec_env_ptr, &locals)
            .expect("fail to gen_check_state_and_snapshot");
        ctx.num_migration_points += 1;
    }

    // compile instructions
    let mut op_reader = f.get_operators_reader()?.get_binary_reader();
    let mut num_op = 0;
    while !op_reader.eof() {
        let op = op_reader.read_operator()?;
        log::trace!("- op[{}]: {:?}", num_op, &op);
        ctx.current_op = Some(num_op);
        compile_op(ctx, &op, &exec_env_ptr, &mut locals)?;

        num_op += 1;
    }

    // finalize dispatcher for restore
    if !ctx.config.no_restore && (ctx.config.enable_cr || ctx.config.legacy_cr) {
        ctx.builder
            .position_at_end(ctx.restore_dispatch_bb.unwrap());
        gen_finalize_restore_dispatch(ctx, &exec_env_ptr)
            .expect("should gen finalize restore dispatch");
    }

    ctx.current_fn = None;
    Ok(())
}

fn compile_op<'a>(
    ctx: &mut Context<'a, '_>,
    op: &Operator,
    exec_env_ptr: &PointerValue<'a>,
    locals: &mut [(PointerValue<'a>, BasicTypeEnum<'a>)],
) -> Result<()> {
    if ctx.unreachable_depth != 0 {
        log::trace!("- under unreachable");
        match op {
            Operator::Block { blockty: _ }
            | Operator::Loop { blockty: _ }
            | Operator::If { blockty: _ } => {
                ctx.unreachable_depth += 1;
                return Ok(());
            }
            Operator::Else => {
                if ctx.unreachable_depth == 1 {
                    gen_else(ctx).context("error gen Else")?;
                    ctx.unreachable_depth -= 1;
                    ctx.unreachable_reason = UnreachableReason::Reachable;
                    log::trace!("- end of unreachable");
                    return Ok(());
                } else {
                    return Ok(());
                }
            }
            Operator::End => match ctx.unreachable_depth {
                0 => {
                    unreachable!("Unexpected depth 0");
                }
                1 => {
                    gen_end(ctx).context("error gen End")?;
                    ctx.unreachable_depth -= 1;
                    ctx.unreachable_reason = UnreachableReason::Reachable;
                    log::trace!("- end of unreachable");
                    return Ok(());
                }
                _ => {
                    ctx.unreachable_depth -= 1;
                    return Ok(());
                }
            },
            _ => {
                return Ok(());
            }
        }
    }

    match &op {
        /******************************
          Control flow instructions
        ******************************/
        Operator::Block { blockty } => {
            gen_block(ctx, blockty).context("error gen Block")?;
        }
        Operator::Loop { blockty } => {
            gen_loop(ctx, exec_env_ptr, locals, blockty).context("error gen Loop")?;
        }
        Operator::If { blockty } => {
            gen_if(ctx, blockty).context("error gen If")?;
        }
        Operator::Else {} => {
            gen_else(ctx).context("error gen Else")?;
        }
        Operator::Br { relative_depth } => {
            gen_br(ctx, *relative_depth).context("error gen Br")?;
        }
        Operator::BrIf { relative_depth } => {
            gen_brif(ctx, *relative_depth).context("errpr gen BrIf")?;
        }
        Operator::BrTable { targets } => {
            gen_br_table(ctx, targets).context("error gen BrTable")?;
        }
        Operator::End => {
            log::trace!(
                "- gen_end, fn = {:?}, ret = {:?}",
                ctx.current_fn.unwrap().get_name(),
                ctx.current_fn.unwrap().get_type().get_return_type()
            );
            gen_end(ctx).context("error gen End")?;
        }
        Operator::Call { function_index } => {
            gen_call(ctx, exec_env_ptr, locals, *function_index).context("error gen Call")?;
        }
        Operator::CallIndirect {
            type_index,
            table_index,
        } => {
            gen_call_indirect(ctx, exec_env_ptr, locals, *type_index, *table_index)
                .context("error gen CallIndirect")?;
        }
        Operator::Drop => {
            gen_drop(ctx).context("error gen Drop")?;
        }
        Operator::Return => {
            gen_return(ctx).context("error gen Return")?;
        }
        Operator::Select => {
            gen_select(ctx).context("error gen Select")?;
        }
        Operator::Nop => {
            // Do nothing
        }
        Operator::Unreachable => {
            gen_unreachable(ctx).context("error gen Unreachable")?;
        }
        /******************************
          Numeric instructions
        ******************************/
        Operator::I32Const { value } => {
            let i = ctx.inkwell_types.i32_type.const_int(*value as u64, false);
            ctx.push(i.as_basic_value_enum());
        }
        Operator::I64Const { value } => {
            let i = ctx.inkwell_types.i64_type.const_int(*value as u64, false);
            ctx.push(i.as_basic_value_enum());
        }
        Operator::F32Const { value } => {
            let bits = ctx
                .inkwell_types
                .i32_type
                .const_int(value.bits() as u64, false);
            let i = ctx
                .builder
                .build_bit_cast(bits, ctx.inkwell_types.f32_type, "")
                .expect("should build bit_cast");
            ctx.push(i);
        }
        Operator::F64Const { value } => {
            let bits = ctx.inkwell_types.i64_type.const_int(value.bits(), false);
            let i = ctx
                .builder
                .build_bit_cast(bits, ctx.inkwell_types.f64_type, "")
                .expect("should build bit_cast");
            ctx.push(i);
        }
        Operator::I32Clz => {
            let v1 = ctx.pop().expect("stack empty");
            gen_llvm_intrinsic(
                ctx,
                ctx.inkwell_intrs.ctlz_i32,
                &[v1.into(), ctx.inkwell_types.bool_type.const_zero().into()],
            )
            .expect("error gen I32Clz");
        }
        Operator::I64Clz => {
            let v1 = ctx.pop().expect("stack empty");
            let function = ctx.inkwell_intrs.ctlz_i64;
            let clz = ctx
                .builder
                .build_call(
                    function,
                    &[v1.into(), ctx.inkwell_types.bool_type.const_zero().into()],
                    "",
                )
                .expect("fail build_call llvm_insts")
                .try_as_basic_value()
                .left()
                .expect("fail build_call llvm_insts");
            let res = ctx
                .builder
                .build_int_sub(
                    ctx.inkwell_types.i64_type.const_int(63, false),
                    clz.into_int_value(),
                    "",
                )
                .expect("fail build_int_sub llvm_insts");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::I32Ctz => {
            let v1 = ctx.pop().expect("stack empty");
            gen_llvm_intrinsic(
                ctx,
                ctx.inkwell_intrs.cttz_i32,
                &[v1.into(), ctx.inkwell_types.bool_type.const_zero().into()],
            )
            .context("error gen I32Ctz")?;
        }
        Operator::I64Ctz => {
            let v1 = ctx.pop().expect("stack empty");
            gen_llvm_intrinsic(
                ctx,
                ctx.inkwell_intrs.cttz_i64,
                &[v1.into(), ctx.inkwell_types.bool_type.const_zero().into()],
            )
            .context("error gen I64Ctz")?;
        }
        Operator::I32Popcnt => {
            let v1 = ctx.pop().expect("stack empty");
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.ctpop_i32, &[v1.into()])
                .expect("error gen I32Popcnt");
        }
        Operator::I64Popcnt => {
            let v1 = ctx.pop().expect("stack empty");
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.ctpop_i64, &[v1.into()])
                .expect("error gen I64Popcnt");
        }
        Operator::I32Add | Operator::I64Add => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_int_add(v1.into_int_value(), v2.into_int_value(), "")
                .expect("should build int add");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::I32Sub | Operator::I64Sub => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_int_sub(v1.into_int_value(), v2.into_int_value(), "")
                .expect("should build int sub");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::I32Mul | Operator::I64Mul => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_int_mul(v1.into_int_value(), v2.into_int_value(), "")
                .expect("should build int mul");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::I32DivS | Operator::I64DivS => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_int_signed_div(v1.into_int_value(), v2.into_int_value(), "")
                .expect("should build int signed div");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::I32DivU | Operator::I64DivU => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_int_unsigned_div(v1.into_int_value(), v2.into_int_value(), "")
                .expect("should build int unsigned div");
            ctx.push(res.as_basic_value_enum());
        }
        /* % operator */
        Operator::I32RemS | Operator::I64RemS => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_int_signed_rem(v1.into_int_value(), v2.into_int_value(), "")
                .expect("should build int signed rem");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::I32RemU | Operator::I64RemU => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_int_unsigned_rem(v1.into_int_value(), v2.into_int_value(), "")
                .expect("should build int unsigned rem");
            ctx.push(res.as_basic_value_enum());
        }
        /******************************
            bitwise instructions
        ******************************/
        Operator::I32And | Operator::I64And => {
            helper::gen_and(ctx).context("error gen And")?;
        }
        Operator::I32Or | Operator::I64Or => {
            helper::gen_or(ctx).context("error gen Or")?;
        }
        Operator::I32Xor | Operator::I64Xor => {
            helper::gen_xor(ctx).context("error gen Xor")?;
        }
        Operator::I32Shl | Operator::I64Shl => {
            helper::gen_shl(ctx).context("error gen Shl")?;
        }
        Operator::I32ShrS | Operator::I64ShrS => {
            helper::gen_shr(ctx, true).context("error gen ShrS")?;
        }
        Operator::I32ShrU | Operator::I64ShrU => {
            helper::gen_shr(ctx, false).context("error gen ShrU")?;
        }
        Operator::I32Rotl => {
            helper::gen_rotl(ctx, true).context("error gen I32Rotl")?;
        }
        Operator::I64Rotl => {
            helper::gen_rotl(ctx, false).context("error gen I64Rotl")?;
        }
        Operator::I32Rotr => {
            helper::gen_rotr(ctx, true).context("error gen I32Rotr")?;
        }
        Operator::I64Rotr => {
            helper::gen_rotr(ctx, false).context("error gen I64Rotr")?;
        }
        /******************************
          Conversion instructions
        ******************************/
        Operator::I32WrapI64 => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let wraped = ctx
                .builder
                .build_int_truncate(v, ctx.inkwell_types.i32_type, "")
                .expect("error build int truncate");
            ctx.push(wraped.as_basic_value_enum());
        }
        Operator::I64Extend32S => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let narrow_value = ctx
                .builder
                .build_int_truncate(v, ctx.inkwell_types.i32_type, "")
                .expect("error build int truncate");
            let extended = ctx
                .builder
                .build_int_s_extend(narrow_value, ctx.inkwell_types.i64_type, "i64extend32s")
                .expect("error build int s extend");
            ctx.push(extended.as_basic_value_enum());
        }
        Operator::I64Extend16S => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let narrow_value = ctx
                .builder
                .build_int_truncate(v, ctx.inkwell_types.i16_type, "")
                .expect("error build int truncate");
            let extended = ctx
                .builder
                .build_int_s_extend(narrow_value, ctx.inkwell_types.i64_type, "i64extend16s")
                .expect("error build int s extend");
            ctx.push(extended.as_basic_value_enum());
        }
        Operator::I64Extend8S => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let narrow_value = ctx
                .builder
                .build_int_truncate(v, ctx.inkwell_types.i8_type, "")
                .expect("error build int truncate");
            let extended = ctx
                .builder
                .build_int_s_extend(narrow_value, ctx.inkwell_types.i64_type, "i64extend8s")
                .expect("error build int s extend");
            ctx.push(extended.as_basic_value_enum());
        }
        Operator::I32Extend16S => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let narrow_value = ctx
                .builder
                .build_int_truncate(v, ctx.inkwell_types.i16_type, "")
                .expect("error build int truncate");
            let extended = ctx
                .builder
                .build_int_s_extend(narrow_value, ctx.inkwell_types.i32_type, "i32extend16s")
                .expect("error build int s extend");
            ctx.push(extended.as_basic_value_enum());
        }
        Operator::I32Extend8S => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let narrow_value = ctx
                .builder
                .build_int_truncate(v, ctx.inkwell_types.i8_type, "")
                .expect("error build int truncate");
            let extended = ctx
                .builder
                .build_int_s_extend(narrow_value, ctx.inkwell_types.i32_type, "i32extend8s")
                .expect("error build int s extend");
            ctx.push(extended.as_basic_value_enum());
        }
        Operator::I64ExtendI32U => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let extended = ctx
                .builder
                .build_int_z_extend(v, ctx.inkwell_types.i64_type, "i64extendi32u")
                .expect("error build int z extend");
            ctx.push(extended.as_basic_value_enum());
        }
        Operator::I64ExtendI32S => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let extended = ctx
                .builder
                .build_int_s_extend(v, ctx.inkwell_types.i64_type, "i64extendi32s")
                .expect("error build int s extend");
            ctx.push(extended.as_basic_value_enum());
        }
        Operator::F32DemoteF64 => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            let demoted = ctx
                .builder
                .build_float_trunc(v, ctx.inkwell_types.f32_type, "f32demotef64")
                .expect("error build float trunc");
            ctx.push(demoted.as_basic_value_enum());
        }
        Operator::F64PromoteF32 => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            let promoted = ctx
                .builder
                .build_float_ext(v, ctx.inkwell_types.f64_type, "f64promotef32")
                .expect("error build float ext");
            ctx.push(promoted.as_basic_value_enum());
        }
        Operator::F64ConvertI64S | Operator::F64ConvertI32S => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let converted = ctx
                .builder
                .build_signed_int_to_float(v, ctx.inkwell_types.f64_type, "f64converti64s")
                .expect("error build signed int to float");
            ctx.push(converted.as_basic_value_enum());
        }
        Operator::F64ConvertI64U | Operator::F64ConvertI32U => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let converted = ctx
                .builder
                .build_unsigned_int_to_float(v, ctx.inkwell_types.f64_type, "f64converti64u")
                .expect("error build unsigned int to float");
            ctx.push(converted.as_basic_value_enum());
        }
        Operator::F32ConvertI32S | Operator::F32ConvertI64S => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let converted = ctx
                .builder
                .build_signed_int_to_float(v, ctx.inkwell_types.f32_type, "f32converti32s")
                .expect("error build signed int to float");
            ctx.push(converted.as_basic_value_enum());
        }
        Operator::F32ConvertI32U | Operator::F32ConvertI64U => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let converted = ctx
                .builder
                .build_unsigned_int_to_float(v, ctx.inkwell_types.f32_type, "f32converti32u")
                .expect("error build unsigned int to float");
            ctx.push(converted.as_basic_value_enum());
        }
        Operator::I64TruncF64S | Operator::I64TruncF32S => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            let converted = ctx
                .builder
                .build_float_to_signed_int(v, ctx.inkwell_types.i64_type, "i64truncf64s")
                .expect("error build float to signed int");
            ctx.push(converted.as_basic_value_enum());
        }
        Operator::I32TruncF32S | Operator::I32TruncF64S => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            let converted = ctx
                .builder
                .build_float_to_signed_int(v, ctx.inkwell_types.i32_type, "i32truncf32s")
                .expect("error build float to signed int");
            ctx.push(converted.as_basic_value_enum());
        }
        Operator::I64TruncF64U | Operator::I64TruncF32U => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            let converted = ctx
                .builder
                .build_float_to_unsigned_int(v, ctx.inkwell_types.i64_type, "i64truncf64u")
                .expect("error build float to unsigned int");
            ctx.push(converted.as_basic_value_enum());
        }
        Operator::I32TruncF32U | Operator::I32TruncF64U => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            let converted = ctx
                .builder
                .build_float_to_unsigned_int(v, ctx.inkwell_types.i32_type, "i32truncf32u")
                .expect("error build float to unsigned int");
            ctx.push(converted.as_basic_value_enum());
        }
        Operator::F64ReinterpretI64 => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let reinterpreted = ctx
                .builder
                .build_bit_cast(v, ctx.inkwell_types.f64_type, "")
                .expect("error build bit_cast");
            ctx.push(reinterpreted);
        }
        Operator::F32ReinterpretI32 => {
            let v = ctx.pop().expect("stack empty").into_int_value();
            let reinterpreted = ctx
                .builder
                .build_bit_cast(v, ctx.inkwell_types.f32_type, "")
                .expect("error build bit_cast");
            ctx.push(reinterpreted);
        }
        Operator::I64ReinterpretF64 => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            let reinterpreted = ctx
                .builder
                .build_bit_cast(v, ctx.inkwell_types.i64_type, "")
                .expect("error build bit_cast");
            ctx.push(reinterpreted);
        }
        Operator::I32ReinterpretF32 => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            let reinterpreted = ctx
                .builder
                .build_bit_cast(v, ctx.inkwell_types.i32_type, "")
                .expect("error build bit_cast");
            ctx.push(reinterpreted);
        }
        /******************************
            Floating
        ******************************/
        Operator::F32Eq | Operator::F64Eq => {
            gen_float_compare(ctx, inkwell::FloatPredicate::OEQ).expect("error gen compare float");
        }
        Operator::F32Ne | Operator::F64Ne => {
            gen_float_compare(ctx, inkwell::FloatPredicate::UNE).expect("error gen compare float");
        }
        Operator::F64Lt | Operator::F32Lt => {
            gen_float_compare(ctx, inkwell::FloatPredicate::OLT).expect("error gen compare float");
        }
        Operator::F64Gt | Operator::F32Gt => {
            gen_float_compare(ctx, inkwell::FloatPredicate::OGT).expect("error gen compare float");
        }
        Operator::F64Le | Operator::F32Le => {
            gen_float_compare(ctx, inkwell::FloatPredicate::OLE).expect("error gen compare float");
        }
        Operator::F64Ge | Operator::F32Ge => {
            gen_float_compare(ctx, inkwell::FloatPredicate::OGE).expect("error gen compare float");
        }
        Operator::F64Abs => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.fabs_f64, &[v.into()])
                .context("error gen F64Abs")?;
        }
        Operator::F32Abs => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.fabs_f32, &[v.into()])
                .context("error gen F32Abs")?;
        }
        Operator::F64Neg => {
            let v1 = ctx.pop().expect("stack empty");
            let res = ctx
                .builder
                .build_float_neg(v1.into_float_value(), "f64neg")
                .expect("should build float neg");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::F32Neg => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            let res = ctx
                .builder
                .build_float_neg(v, "f32neg")
                .expect("should build float neg");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::F64Ceil => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.ceil_f64, &[v.into()])
                .context("error gen F64Ceil")?;
        }
        Operator::F32Ceil => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.ceil_f32, &[v.into()])
                .context("error gen F32Ceil")?;
        }
        Operator::F64Floor => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.floor_f64, &[v.into()])
                .context("error gen F64Floor")?;
        }
        Operator::F32Floor => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.floor_f32, &[v.into()])
                .context("error gen F32Floor")?;
        }
        Operator::F64Trunc => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.trunc_f64, &[v.into()])
                .context("error gen F64Trunc")?;
        }
        Operator::F32Trunc => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.trunc_f32, &[v.into()])
                .context("error gen F32Trunc")?;
        }
        Operator::F64Nearest => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.nearbyint_f64, &[v.into()])
                .context("error gen F64Nearest")?;
        }
        Operator::F32Nearest => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.nearbyint_f32, &[v.into()])
                .context("error gen F32Nearest")?;
        }
        Operator::F64Sqrt => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.sqrt_f64, &[v.into()])
                .context("error gen F64Sqrt")?;
        }
        Operator::F32Sqrt => {
            let v = ctx.pop().expect("stack empty").into_float_value();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.sqrt_f32, &[v.into()])
                .context("error gen F64Sqrt")?;
        }
        Operator::F64Add | Operator::F32Add => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_float_add(v1.into_float_value(), v2.into_float_value(), "")
                .expect("should build float add");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::F64Sub | Operator::F32Sub => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_float_sub(v1.into_float_value(), v2.into_float_value(), "")
                .expect("should build float sub");
            ctx.push(res.as_basic_value_enum());
        }
        Operator::F64Mul | Operator::F32Mul => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_float_mul(v1.into_float_value(), v2.into_float_value(), "")
                .expect("should build float mul");
            ctx.push(res.as_basic_value_enum());
        }

        Operator::F64Div | Operator::F32Div => {
            let (v1, v2) = ctx.pop2();
            let res = ctx
                .builder
                .build_float_div(v1.into_float_value(), v2.into_float_value(), "")
                .expect("should build float div");
            ctx.push(res.as_basic_value_enum());
        }

        Operator::F64Min => {
            let (v1, v2) = ctx.pop2();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.minnum_f64, &[v1.into(), v2.into()])
                .context("error gen F64Min")?;
        }
        Operator::F32Min => {
            let (v1, v2) = ctx.pop2();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.minnum_f32, &[v1.into(), v2.into()])
                .context("error gen F32Min")?;
        }
        Operator::F64Max => {
            let (v1, v2) = ctx.pop2();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.maxnum_f64, &[v1.into(), v2.into()])
                .context("error gen F64Max")?;
        }
        Operator::F32Max => {
            let (v1, v2) = ctx.pop2();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.maxnum_f32, &[v1.into(), v2.into()])
                .context("error gen F32Max")?;
        }
        Operator::F32Copysign => {
            let (v1, v2) = ctx.pop2();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.copysign_f32, &[v1.into(), v2.into()])
                .context("error gen copysign.f32")?;
        }
        Operator::F64Copysign => {
            let (v1, v2) = ctx.pop2();
            gen_llvm_intrinsic(ctx, ctx.inkwell_intrs.copysign_f64, &[v1.into(), v2.into()])
                .context("error gen copysign.f64")?;
        }
        /******************************
          Variables
        ******************************/
        // Loads the value of local variable to stack
        Operator::LocalGet { local_index } => {
            assert!(*local_index < locals.len() as u32);
            let (value_ptr, ty) = locals[*local_index as usize];
            let v = ctx
                .builder
                .build_load(ty, value_ptr, "")
                .expect("should build load");
            ctx.push(v);
        }
        // Sets the value of the local variable
        Operator::LocalSet { local_index } => {
            assert!(*local_index < locals.len() as u32);
            let (value_ptr, _) = locals[*local_index as usize];
            let v = ctx.pop().expect("stack empty");
            ctx.builder
                .build_store(value_ptr, v)
                .expect("should build store");
        }
        Operator::LocalTee { local_index } => {
            assert!(*local_index < locals.len() as u32);
            let (value_ptr, _) = locals[*local_index as usize];
            let v = ctx.pop().expect("stack empty");
            ctx.builder
                .build_store(value_ptr, v)
                .expect("should build store");
            ctx.push(v);
        }
        Operator::GlobalGet { global_index } => {
            assert!(*global_index < ctx.globals.len() as u32);
            let global = &ctx.globals[*global_index as usize];
            match global {
                Global::Const { value } => {
                    ctx.push(*value);
                }
                Global::Mut { ptr, ty } => {
                    let value = ctx
                        .builder
                        .build_load(*ty, ptr.as_pointer_value(), "")
                        .expect("should build load");
                    ctx.push(value);
                }
            };
        }
        Operator::GlobalSet { global_index } => {
            assert!(*global_index < ctx.globals.len() as u32);
            let global = &ctx.globals[*global_index as usize];
            match global {
                Global::Const { value: _ } => {
                    bail!("Global.Set to const value");
                }
                Global::Mut { ptr, ty: _ } => {
                    let value = ctx
                        .stack_frames
                        .last_mut()
                        .expect("frame empty")
                        .stack
                        .pop()
                        .expect("stack empty");
                    ctx.builder
                        .build_store(ptr.as_pointer_value(), value)
                        .expect("should build store");
                }
            };
        }
        /******************************
          Memory instructions
        ******************************/
        Operator::MemorySize { mem: _ } => {
            compile_op_memory_size(ctx, exec_env_ptr).context("error gen MemorySize")?;
        }
        Operator::MemoryGrow { mem: _ } => {
            compile_op_memory_grow(ctx, exec_env_ptr).context("error gen MemoryGrow")?;
        }
        Operator::MemoryCopy { dst_mem, src_mem } => {
            compile_op_memcpy(ctx, exec_env_ptr, *dst_mem, *src_mem)
                .context("error gen MemoryCopy")?;
        }
        Operator::MemoryFill { mem } => {
            compile_op_memory_fill(ctx, exec_env_ptr, *mem).context("error gen MemoryFill")?;
        }
        // TODO: memarg
        Operator::I32Load { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                false,
                false,
            )
            .context("error gen I32Load")?;
        }
        Operator::I64Load { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i64_type.as_basic_type_enum(),
                ctx.inkwell_types.i64_type.as_basic_type_enum(),
                false,
                false,
            )
            .context("error gen I64Load")?;
        }
        Operator::F32Load { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.f32_type.as_basic_type_enum(),
                ctx.inkwell_types.f32_type.as_basic_type_enum(),
                false,
                false,
            )
            .context("error gen F32Load")?;
        }
        Operator::F64Load { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.f64_type.as_basic_type_enum(),
                ctx.inkwell_types.f64_type.as_basic_type_enum(),
                false,
                false,
            )
            .context("error gen F64Load")?;
        }
        Operator::I32Load8S { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                ctx.inkwell_types.i8_type.as_basic_type_enum(),
                true,
                true,
            )
            .context("error gen I32Load8S")?;
        }
        Operator::I32Load8U { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                ctx.inkwell_types.i8_type.as_basic_type_enum(),
                false,
                true,
            )
            .context("error gen I32Load8U")?;
        }
        Operator::I32Load16S { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                ctx.inkwell_types.i16_type.as_basic_type_enum(),
                true,
                true,
            )
            .context("error gen I32Load16S")?;
        }
        Operator::I32Load16U { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                ctx.inkwell_types.i16_type.as_basic_type_enum(),
                false,
                true,
            )
            .context("error gen I32Load16S")?;
        }
        Operator::I64Load8S { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i64_type.as_basic_type_enum(),
                ctx.inkwell_types.i8_type.as_basic_type_enum(),
                true,
                true,
            )
            .context("error gen I64Load8S")?;
        }
        Operator::I64Load8U { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i64_type.as_basic_type_enum(),
                ctx.inkwell_types.i8_type.as_basic_type_enum(),
                false,
                true,
            )
            .context("error gen I64Load8U")?;
        }
        Operator::I64Load16S { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i64_type.as_basic_type_enum(),
                ctx.inkwell_types.i16_type.as_basic_type_enum(),
                true,
                true,
            )
            .context("error gen I64Load16S")?;
        }
        Operator::I64Load16U { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i64_type.as_basic_type_enum(),
                ctx.inkwell_types.i16_type.as_basic_type_enum(),
                false,
                true,
            )
            .context("error gen I64Load16U")?;
        }
        Operator::I64Load32S { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i64_type.as_basic_type_enum(),
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                true,
                true,
            )
            .context("error gen I64Load32S")?;
        }
        Operator::I64Load32U { memarg } => {
            compile_op_load(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i64_type.as_basic_type_enum(),
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                false,
                true,
            )
            .context("error gen I64Load32U")?;
        }
        Operator::I32Store { memarg } => {
            compile_op_store(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                false,
            )
            .context("error gen I32Store")?;
        }
        Operator::I64Store { memarg } => {
            compile_op_store(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i64_type.as_basic_type_enum(),
                false,
            )
            .context("error gen I64Store")?;
        }
        Operator::F32Store { memarg } => {
            compile_op_store(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.f32_type.as_basic_type_enum(),
                false,
            )
            .context("error gen F32Store")?;
        }
        Operator::F64Store { memarg } => {
            compile_op_store(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.f64_type.as_basic_type_enum(),
                false,
            )
            .context("error gen F64Store")?;
        }
        Operator::I32Store8 { memarg } | Operator::I64Store8 { memarg } => {
            compile_op_store(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i8_type.as_basic_type_enum(),
                true,
            )
            .context("error I32Store")?;
        }
        Operator::I32Store16 { memarg } | Operator::I64Store16 { memarg } => {
            compile_op_store(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i16_type.as_basic_type_enum(),
                true,
            )
            .context("error I32Store16")?;
        }
        Operator::I64Store32 { memarg } => {
            compile_op_store(
                ctx,
                exec_env_ptr,
                memarg,
                ctx.inkwell_types.i32_type.as_basic_type_enum(),
                true,
            )
            .context("error gen I64Store32")?;
        }
        /******************************
          Comparison instructions
        ******************************/
        Operator::I32Eqz => {
            ctx.push(
                ctx.inkwell_types
                    .i32_type
                    .const_zero()
                    .as_basic_value_enum(),
            );
            gen_int_compare(ctx, inkwell::IntPredicate::EQ).context("error gen I32Eqz")?;
        }
        Operator::I64Eqz => {
            ctx.push(
                ctx.inkwell_types
                    .i64_type
                    .const_zero()
                    .as_basic_value_enum(),
            );
            gen_int_compare(ctx, inkwell::IntPredicate::EQ).context("error gen I64Eqz")?;
        }
        Operator::I32Eq | Operator::I64Eq => {
            gen_int_compare(ctx, inkwell::IntPredicate::EQ).context("error gen Eq")?;
        }
        Operator::I32Ne | Operator::I64Ne => {
            gen_int_compare(ctx, inkwell::IntPredicate::NE).context("error gen Ne")?;
        }
        Operator::I32LtS | Operator::I64LtS => {
            gen_int_compare(ctx, inkwell::IntPredicate::SLT).context("error gen LtS")?;
        }
        Operator::I32LtU | Operator::I64LtU => {
            gen_int_compare(ctx, inkwell::IntPredicate::ULT).context("error gen LtU")?;
        }
        Operator::I32GtS | Operator::I64GtS => {
            gen_int_compare(ctx, inkwell::IntPredicate::SGT).context("error gen GtS")?;
        }
        Operator::I32GtU | Operator::I64GtU => {
            gen_int_compare(ctx, inkwell::IntPredicate::UGT).context("error gen GtU")?;
        }
        Operator::I32LeS | Operator::I64LeS => {
            gen_int_compare(ctx, inkwell::IntPredicate::SLE).context("error gen LeS")?;
        }
        Operator::I32LeU | Operator::I64LeU => {
            gen_int_compare(ctx, inkwell::IntPredicate::ULE).context("error gen LeU")?;
        }
        Operator::I32GeS | Operator::I64GeS => {
            gen_int_compare(ctx, inkwell::IntPredicate::SGE).context("error gen GeS")?;
        }
        Operator::I32GeU | Operator::I64GeU => {
            gen_int_compare(ctx, inkwell::IntPredicate::UGE).context("error gen GeU")?;
        }
        _ => {
            log::error!("Unimplemented instruction {:?}", op);
            bail!("Unimplemented instruction {:?}", op);
        }
    }
    Ok(())
}

pub fn compile_op_memory_size<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<()> {
    let size = gen_memory_size(ctx, exec_env_ptr).expect("error gen memory size");
    ctx.push(size);
    Ok(())
}

pub fn compile_op_memory_grow<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<()> {
    let delta = ctx.pop().expect("stack empty");
    let ret = ctx
        .builder
        .build_call(
            ctx.fn_memory_grow.expect("shold define fn_memory_grow"),
            &[exec_env_ptr.as_basic_value_enum().into(), delta.into()],
            "memory_grow",
        )
        .expect("should build call")
        .as_any_value_enum()
        .into_int_value()
        .as_basic_value_enum();
    ctx.push(ret);
    Ok(())
}

pub fn compile_op_memcpy<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    dst_mem: u32,
    src_mem: u32,
) -> Result<()> {
    // TODO: multi memory
    assert_eq!(dst_mem, 0);
    assert_eq!(src_mem, 0);

    let len = ctx.pop().expect("stack empty");
    let src = ctx.pop().expect("stack empty");
    let dst = ctx.pop().expect("stack empty");
    let src_addr = resolve_pointer(ctx, exec_env_ptr, src.into_int_value());
    let dst_addr = resolve_pointer(ctx, exec_env_ptr, dst.into_int_value());
    ctx.builder
        .build_memcpy(dst_addr, 1, src_addr, 1, len.into_int_value())
        .map_err(|e| anyhow!(e))
        .context("error build_memcpy")?;
    Ok(())
}

pub fn compile_op_memory_fill<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    mem: u32,
) -> Result<()> {
    // TODO: multi memory
    assert_eq!(mem, 0);

    let len = ctx.pop().expect("stack empty");
    let val = ctx.pop().expect("stack empty");
    let dst = ctx.pop().expect("stack empty");
    let dst_addr = resolve_pointer(ctx, exec_env_ptr, dst.into_int_value());
    let val_i8 = ctx
        .builder
        .build_int_truncate(val.into_int_value(), ctx.inkwell_types.i8_type, "val_i8")
        .expect("error build int truncate");
    ctx.builder
        .build_memset(dst_addr, 1, val_i8, len.into_int_value())
        .map_err(|e| anyhow!(e))
        .context("error build_memset")?;
    Ok(())
}

fn resolve_pointer<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    offset: IntValue<'a>,
) -> PointerValue<'a> {
    let memory_base = gen_memory_base(ctx, exec_env_ptr).expect("error gen memory base");
    // calculate base + offset
    let dst_addr = unsafe {
        ctx.builder.build_gep(
            ctx.inkwell_types.i8_type,
            memory_base,
            &[offset],
            "resolved_addr",
        )
    }
    .expect("should build gep");
    // cast pointer value
    ctx.builder
        .build_bit_cast(dst_addr, ctx.inkwell_types.ptr_type, "bit_casted")
        .expect("should build bit_cast")
        .into_pointer_value()
}

pub fn compile_op_load<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    memarg: &MemArg,
    extended_type: inkwell::types::BasicTypeEnum<'a>,
    load_type: inkwell::types::BasicTypeEnum<'a>,
    signed: bool,
    require_extend: bool,
) -> Result<()> {
    // offset
    let address_operand = ctx.pop().expect("stack empty").into_int_value();
    let address_operand_ex = ctx
        .builder
        .build_int_z_extend(address_operand, ctx.inkwell_types.i64_type, "")
        .expect("error build int z extend");
    let memarg_offset = ctx.inkwell_types.i64_type.const_int(memarg.offset, false);
    let offset = ctx
        .builder
        .build_int_add(address_operand_ex, memarg_offset, "offset")
        .expect("error build int add");

    // get actual virtual address
    let dst_addr = resolve_pointer(ctx, exec_env_ptr, offset);
    // load value
    let result = ctx
        .builder
        .build_load(load_type, dst_addr, "loaded")
        .expect("error build load");

    // push loaded value
    if require_extend {
        // extend value
        let extended_result = match signed {
            true => ctx
                .builder
                .build_int_s_extend(
                    result.into_int_value(),
                    extended_type.into_int_type(),
                    "loaded_extended",
                )
                .expect("error build int s extend"),
            false => ctx
                .builder
                .build_int_z_extend(
                    result.into_int_value(),
                    extended_type.into_int_type(),
                    "loaded_extended",
                )
                .expect("error build int z extend"),
        };
        ctx.push(extended_result.as_basic_value_enum());
    } else {
        ctx.push(result.as_basic_value_enum());
    }
    Ok(())
}

pub fn compile_op_store<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    memarg: &MemArg,
    store_type: inkwell::types::BasicTypeEnum<'a>,
    require_narrow: bool,
) -> Result<()> {
    // value
    let value = ctx.pop().expect("stack empty");

    // offset
    let address_operand = ctx.pop().expect("stack empty").into_int_value();
    let address_operand_ex = ctx
        .builder
        .build_int_z_extend(address_operand, ctx.inkwell_types.i64_type, "")
        .expect("error build int z extend");
    let memarg_offset = ctx.inkwell_types.i64_type.const_int(memarg.offset, false);
    let offset = ctx
        .builder
        .build_int_add(address_operand_ex, memarg_offset, "offset")
        .expect("error build int add");

    // get actual virtual address
    let dst_addr = resolve_pointer(ctx, exec_env_ptr, offset);

    if require_narrow {
        let narrow_value = ctx
            .builder
            .build_int_truncate(
                value.into_int_value(),
                store_type.into_int_type(),
                "narrow_value",
            )
            .expect("error build int truncate");
        ctx.builder
            .build_store(dst_addr, narrow_value)
            .expect("should build store");
    } else {
        ctx.builder
            .build_store(dst_addr, value)
            .expect("should build store");
    }

    Ok(())
}
