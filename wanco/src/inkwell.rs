use inkwell::{context::Context, module::Module, types::*, values::FunctionValue, AddressSpace};

pub struct InkwellTypes<'ctx> {
    // primitive types
    pub void_type: VoidType<'ctx>,
    pub bool_type: IntType<'ctx>,
    pub i8_type: IntType<'ctx>,
    pub i16_type: IntType<'ctx>,
    pub i32_type: IntType<'ctx>,
    pub i64_type: IntType<'ctx>,
    pub f32_type: FloatType<'ctx>,
    pub f64_type: FloatType<'ctx>,

    // pointer types
    pub i8_ptr_type: PointerType<'ctx>,
    pub i16_ptr_type: PointerType<'ctx>,
    pub i32_ptr_type: PointerType<'ctx>,
    pub i64_ptr_type: PointerType<'ctx>,
    pub f32_ptr_type: PointerType<'ctx>,
    pub f64_ptr_type: PointerType<'ctx>,
}

/// Basic insts of inkwell.
pub struct InkwellIntrinsics<'ctx> {
    // llvm intrinsics
    pub ctlz_i32: FunctionValue<'ctx>,
    pub ctlz_i64: FunctionValue<'ctx>,
    pub cttz_i32: FunctionValue<'ctx>,
    pub cttz_i64: FunctionValue<'ctx>,
    pub ctpop_i32: FunctionValue<'ctx>,
    pub ctpop_i64: FunctionValue<'ctx>,
    pub fabs_f32: FunctionValue<'ctx>,
    pub fabs_f64: FunctionValue<'ctx>,
    pub ceil_f32: FunctionValue<'ctx>,
    pub ceil_f64: FunctionValue<'ctx>,
    pub floor_f32: FunctionValue<'ctx>,
    pub floor_f64: FunctionValue<'ctx>,
    pub trunc_f32: FunctionValue<'ctx>,
    pub trunc_f64: FunctionValue<'ctx>,
    pub nearbyint_f32: FunctionValue<'ctx>,
    pub nearbyint_f64: FunctionValue<'ctx>,
    pub sqrt_f32: FunctionValue<'ctx>,
    pub sqrt_f64: FunctionValue<'ctx>,
    pub minnum_f32: FunctionValue<'ctx>,
    pub minnum_f64: FunctionValue<'ctx>,
    pub maxnum_f32: FunctionValue<'ctx>,
    pub maxnum_f64: FunctionValue<'ctx>,
    pub copysign_f32: FunctionValue<'ctx>,
    pub copysign_f64: FunctionValue<'ctx>,

    pub experimental_stackmap: FunctionValue<'ctx>,
}

pub fn init_inkwell<'a>(
    ctx: &'a Context,
    module: &Module<'a>,
) -> (InkwellTypes<'a>, InkwellIntrinsics<'a>) {
    let void_type = ctx.void_type();
    let bool_type = ctx.bool_type();
    let i8_type = ctx.i8_type();
    let i16_type = ctx.i16_type();
    let i32_type = ctx.i32_type();
    let i64_type = ctx.i64_type();
    let f32_type = ctx.f32_type();
    let f64_type = ctx.f64_type();

    let i8_ptr_type = i8_type.ptr_type(AddressSpace::default());
    let i16_ptr_type = i16_type.ptr_type(AddressSpace::default());
    let i32_ptr_type = i32_type.ptr_type(AddressSpace::default());
    let i64_ptr_type = i64_type.ptr_type(AddressSpace::default());
    let f32_ptr_type = f32_type.ptr_type(AddressSpace::default());
    let f64_ptr_type = f64_type.ptr_type(AddressSpace::default());

    let bool_type_meta: BasicMetadataTypeEnum = bool_type.into();
    //let i8_type_meta: BasicMetadataTypeEnum = i8_type.into();
    //let i16_type_meta: BasicMetadataTypeEnum = i16_type.into();
    let i32_type_meta: BasicMetadataTypeEnum = i32_type.into();
    let i64_type_meta: BasicMetadataTypeEnum = i64_type.into();
    let f32_type_meta: BasicMetadataTypeEnum = f32_type.into();
    let f64_type_meta: BasicMetadataTypeEnum = f64_type.into();

    let i32bool_i32 = i32_type.fn_type(&[i32_type_meta, bool_type_meta], false);
    let i64bool_i64 = i64_type.fn_type(&[i64_type_meta, bool_type_meta], false);
    let i32_i32 = i32_type.fn_type(&[i32_type_meta], false);
    let i64_i64 = i64_type.fn_type(&[i64_type_meta], false);
    let f64_f64 = f64_type.fn_type(&[f64_type_meta], false);
    let f32_f32 = f32_type.fn_type(&[f32_type_meta], false);
    let f32f32_f32 = f32_type.fn_type(&[f32_type_meta, f32_type_meta], false);
    let f64f64_f64 = f64_type.fn_type(&[f64_type_meta, f64_type_meta], false);

    let ctlz_i32 = module.add_function("llvm.ctlz.i32", i32bool_i32, None);
    let ctlz_i64 = module.add_function("llvm.ctlz.i64", i64bool_i64, None);
    let cttz_i32 = module.add_function("llvm.cttz.i32", i32bool_i32, None);
    let cttz_i64 = module.add_function("llvm.cttz.i64", i64bool_i64, None);
    let ctpop_i32 = module.add_function("llvm.ctpop.i32", i32_i32, None);
    let ctpop_i64 = module.add_function("llvm.ctpop.i64", i64_i64, None);
    let fabs_f32 = module.add_function("llvm.fabs.f32", f32_f32, None);
    let fabs_f64 = module.add_function("llvm.fabs.f64", f64_f64, None);
    let ceil_f32 = module.add_function("llvm.ceil.f32", f32_f32, None);
    let ceil_f64 = module.add_function("llvm.ceil.f64", f64_f64, None);
    let trunc_f32 = module.add_function("llvm.trunc.f32", f32_f32, None);
    let trunc_f64 = module.add_function("llvm.trunc.f64", f64_f64, None);
    let nearbyint_f32 = module.add_function("llvm.nearbyint.f32", f32_f32, None);
    let nearbyint_f64 = module.add_function("llvm.nearbyint.f64", f64_f64, None);
    let floor_f32 = module.add_function("llvm.floor.f32", f32_f32, None);
    let floor_f64 = module.add_function("llvm.floor.f64", f64_f64, None);
    let sqrt_f32 = module.add_function("llvm.sqrt.f32", f32_f32, None);
    let sqrt_f64 = module.add_function("llvm.sqrt.f64", f64_f64, None);
    let minnum_f32 = module.add_function("llvm.minnum.f32", f32f32_f32, None);
    let minnum_f64 = module.add_function("llvm.minnum.f64", f64f64_f64, None);
    let maxnum_f32 = module.add_function("llvm.maxnum.f32", f32f32_f32, None);
    let maxnum_f64 = module.add_function("llvm.maxnum.f64", f64f64_f64, None);
    let copysign_f32 = module.add_function("llvm.copysign.f32", f32f32_f32, None);
    let copysign_f64 = module.add_function("llvm.copysign.f64", f64f64_f64, None);

    let experimental_stackmap = module.add_function(
        "llvm.experimental.stackmap",
        void_type.fn_type(&[i64_type_meta, i32_type_meta], true),
        None,
    );

    (
        InkwellTypes {
            void_type,
            bool_type,
            i8_type,
            i16_type,
            i32_type,
            i64_type,
            f32_type,
            f64_type,
            i8_ptr_type,
            i16_ptr_type,
            i32_ptr_type,
            i64_ptr_type,
            f32_ptr_type,
            f64_ptr_type,
        },
        InkwellIntrinsics {
            ctlz_i32,
            ctlz_i64,
            cttz_i32,
            cttz_i64,
            ctpop_i32,
            ctpop_i64,
            fabs_f32,
            fabs_f64,
            ceil_f32,
            ceil_f64,
            floor_f32,
            floor_f64,
            trunc_f32,
            trunc_f64,
            nearbyint_f32,
            nearbyint_f64,
            sqrt_f32,
            sqrt_f64,
            minnum_f32,
            minnum_f64,
            maxnum_f32,
            maxnum_f64,
            copysign_f32,
            copysign_f64,
            experimental_stackmap,
        },
    )
}
