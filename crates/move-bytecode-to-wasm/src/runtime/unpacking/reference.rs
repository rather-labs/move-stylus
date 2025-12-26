use crate::{
    CompilationContext,
    abi_types::error::{AbiError, AbiUnpackError},
    abi_types::unpacking::Unpackable,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
};
use walrus::{FunctionBuilder, FunctionId, Module, ValType, ir::MemArg};

pub fn unpack_reference_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut function_body = function_builder.func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    match itype {
        // If inner is a heap type, forward the pointer
        IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress
        | IntermediateType::ISigner
        | IntermediateType::IVector(_)
        | IntermediateType::IStruct { .. }
        | IntermediateType::IGenericStructInstance { .. }
        | IntermediateType::IEnum { .. }
        | IntermediateType::IGenericEnumInstance { .. } => {
            itype.add_unpack_instructions(
                &mut function_body,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            )?;
        }
        // For immediates, allocate and store
        IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64
        | IntermediateType::IBool => {
            let ptr_local = module.locals.add(walrus::ValType::I32);

            let data_size = itype.wasm_memory_data_size()?;
            function_body
                .i32_const(data_size)
                .call(compilation_ctx.allocator)
                .local_tee(ptr_local);

            itype.add_unpack_instructions(
                &mut function_body,
                module,
                reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            )?;

            function_body.store(
                compilation_ctx.memory_id,
                itype.store_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            function_body.local_get(ptr_local);
        }

        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            return Err(RuntimeFunctionError::from(AbiError::from(
                AbiUnpackError::RefInsideRef,
            )));
        }
        IntermediateType::ITypeParameter(_) => {
            return Err(RuntimeFunctionError::from(AbiError::from(
                AbiUnpackError::UnpackingGenericTypeParameter,
            )));
        }
    }

    function_builder.name(RuntimeFunction::UnpackReference.name().to_owned());
    Ok(function_builder.finish(
        vec![reader_pointer, calldata_reader_pointer],
        &mut module.funcs,
    ))
}
