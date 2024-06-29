use crate::context::Context;
use anyhow::Result;
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, PointerValue,
};

pub fn gen_llvm_intrinsic<'a>(
    ctx: &mut Context<'a, '_>,
    function: FunctionValue<'a>,
    args: &[BasicMetadataValueEnum<'a>],
) -> Result<()> {
    let res = ctx
        .builder
        .build_call(function, args, "")
        .expect("should build call to llvm intrinsic")
        .try_as_basic_value()
        .left()
        .expect("should be basic value");
    ctx.push(res);
    Ok(())
}

pub fn gen_exec_env_field_ptr<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
    field_name: &str,
) -> Result<PointerValue<'a>> {
    let exec_env_type = ctx.exec_env_type.expect("should define exec_env");
    let field_idx = *ctx
        .exec_env_fields
        .get(field_name)
        .expect("should define field");
    let field_ptr = ctx
        .builder
        .build_struct_gep(exec_env_type, *exec_env_ptr, field_idx, field_name)
        .expect("should build struct gep");
    Ok(field_ptr)
}

pub fn gen_memory_base<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<PointerValue<'a>> {
    let memory_base_ptr = gen_exec_env_field_ptr(ctx, exec_env_ptr, "memory_base")
        .expect("should gen memory_base ptr");
    let memory_base = ctx
        .builder
        .build_load(
            ctx.inkwell_types.i8_ptr_type,
            memory_base_ptr,
            "memory_base",
        )
        .expect("should build load");
    Ok(memory_base.into_pointer_value())
}

pub fn gen_memory_size<'a>(
    ctx: &mut Context<'a, '_>,
    exec_env_ptr: &PointerValue<'a>,
) -> Result<BasicValueEnum<'a>> {
    let memory_size_ptr = gen_exec_env_field_ptr(ctx, exec_env_ptr, "memory_size")
        .expect("should gen memory_size ptr");
    let memory_size = ctx
        .builder
        .build_load(ctx.inkwell_types.i32_type, memory_size_ptr, "memory_size")
        .expect("should build load");
    Ok(memory_size)
}

pub fn gen_and(ctx: &mut Context<'_, '_>) -> Result<()> {
    let (v1, v2) = ctx.pop2();
    let res = ctx
        .builder
        .build_and(v1.into_int_value(), v2.into_int_value(), "")
        .expect("should build and");
    ctx.push(res.as_basic_value_enum());
    Ok(())
}

pub fn gen_or(ctx: &mut Context<'_, '_>) -> Result<()> {
    let (v1, v2) = ctx.pop2();
    let res = ctx
        .builder
        .build_or(v1.into_int_value(), v2.into_int_value(), "")
        .expect("should build or");
    ctx.push(res.as_basic_value_enum());
    Ok(())
}

pub fn gen_xor(ctx: &mut Context<'_, '_>) -> Result<()> {
    let (v1, v2) = ctx.pop2();
    let res = ctx
        .builder
        .build_xor(v1.into_int_value(), v2.into_int_value(), "")
        .expect("should build xor");
    ctx.push(res.as_basic_value_enum());
    Ok(())
}

pub fn gen_shl(ctx: &mut Context<'_, '_>) -> Result<()> {
    let (v1, v2) = ctx.pop2();
    let res = ctx
        .builder
        .build_left_shift(v1.into_int_value(), v2.into_int_value(), "")
        .expect("should build shl");
    ctx.push(res.as_basic_value_enum());
    Ok(())
}

pub fn gen_shr(ctx: &mut Context<'_, '_>, sign_extend: bool) -> Result<()> {
    let (v1, v2) = ctx.pop2();
    let res = ctx
        .builder
        .build_right_shift(v1.into_int_value(), v2.into_int_value(), sign_extend, "")
        .expect("should build shr");
    ctx.push(res.as_basic_value_enum());
    Ok(())
}

pub fn gen_rotl(ctx: &mut Context<'_, '_>, if_32bit: bool) -> Result<()> {
    let (v1, v2) = ctx.pop2();
    let (v1, v2) = (v1.into_int_value(), v2.into_int_value());
    let mask = if if_32bit {
        ctx.inkwell_types.i32_type.const_int(31u64, false)
    } else {
        ctx.inkwell_types.i64_type.const_int(63u64, false)
    };
    let v2 = ctx.builder.build_and(v2, mask, "")?;
    let lhs = ctx.builder.build_left_shift(v1, v2, "")?;

    let rhs = {
        let negv2 = ctx.builder.build_int_neg(v2, "")?;
        let rhs = ctx.builder.build_and(negv2, mask, "")?;
        ctx.builder.build_right_shift(v1, rhs, false, "")?
    };

    let res = ctx.builder.build_or(lhs, rhs, "")?;
    ctx.push(res.as_basic_value_enum());
    Ok(())
}

pub fn gen_rotr(ctx: &mut Context<'_, '_>, if_32bit: bool) -> Result<()> {
    let (v1, v2) = ctx.pop2();
    let (v1, v2) = (v1.into_int_value(), v2.into_int_value());
    let mask = if if_32bit {
        ctx.inkwell_types.i32_type.const_int(31u64, false)
    } else {
        ctx.inkwell_types.i64_type.const_int(63u64, false)
    };
    let v2 = ctx.builder.build_and(v2, mask, "")?;
    let lhs = ctx.builder.build_right_shift(v1, v2, false, "")?;

    let rhs = {
        let negv2 = ctx.builder.build_int_neg(v2, "")?;
        let rhs = ctx.builder.build_and(negv2, mask, "")?;
        ctx.builder.build_left_shift(v1, rhs, "")?
    };

    let res = ctx.builder.build_or(lhs, rhs, "")?;
    ctx.push(res.as_basic_value_enum());
    Ok(())
}

pub fn gen_float_compare(ctx: &mut Context<'_, '_>, cond: inkwell::FloatPredicate) -> Result<()> {
    let v2 = ctx.pop().expect("stack empty").into_float_value();
    let v1 = ctx.pop().expect("stack empty").into_float_value();
    let cond = ctx
        .builder
        .build_float_compare(cond, v1, v2, "")
        .expect("should build float compare");
    let result = ctx
        .builder
        .build_int_z_extend(cond, ctx.inkwell_types.i32_type, "")
        .expect("should build int z extend");
    ctx.push(result.as_basic_value_enum());

    Ok(())
}

pub fn gen_int_compare(ctx: &mut Context<'_, '_>, cond: inkwell::IntPredicate) -> Result<()> {
    let v2 = ctx.pop().expect("stack empty").into_int_value();
    let v1 = ctx.pop().expect("stack empty").into_int_value();
    let cond = ctx
        .builder
        .build_int_compare(cond, v1, v2, "")
        .expect("should build int compare");
    let result = ctx
        .builder
        .build_int_z_extend(cond, ctx.inkwell_types.i32_type, "")
        .expect("should build int z extend");
    ctx.push(result.as_basic_value_enum());

    Ok(())
}
