use crate::{
    CompilationContext,
    abi_types::error::{AbiError, AbiUnpackError},
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
};
use alloy_sol_types::{SolType, sol_data};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, MemArg, StoreKind},
};

pub fn unpack_enum_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder =
        FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut function_body = function_builder.func_body();

    let enum_ = compilation_ctx.get_enum_by_intermediate_type(itype)?;
    if !enum_.is_simple {
        return Err(
            AbiError::from(AbiUnpackError::EnumIsNotSimple(enum_.identifier.to_owned())).into(),
        );
    }
    let reader_pointer = module.locals.add(ValType::I32);
    let encoded_size =
        sol_data::Uint::<8>::ENCODED_SIZE.ok_or(AbiError::UnableToGetTypeAbiSize)? as i32;

    let unpack_u32_function = RuntimeFunction::UnpackU32.get(module, Some(compilation_ctx))?;

    // Save the variant to check it later
    let variant_number = module.locals.add(ValType::I32);
    function_body
        .local_get(reader_pointer)
        .i32_const(encoded_size)
        .call(unpack_u32_function)
        .local_tee(variant_number);

    // Trap if the variant number is higher that the quantity of variants the enum contains
    function_body
        .i32_const(enum_.variants.len() as i32 - 1)
        .binop(BinaryOp::I32GtU)
        .if_else(
            None,
            |then| {
                then.unreachable();
            },
            |_| {},
        );

    // The enum should occupy only 4 bytes since only the variant number is saved
    let enum_ptr = module.locals.add(ValType::I32);
    function_body
        .i32_const(4)
        .call(compilation_ctx.allocator)
        .local_tee(enum_ptr)
        .local_get(variant_number)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    function_body.local_get(enum_ptr);

    function_builder.name(RuntimeFunction::UnpackEnum.name().to_owned());
    Ok(function_builder.finish(vec![reader_pointer], &mut module.funcs))
}
