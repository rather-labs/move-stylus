use std::rc::Rc;

use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    abi_types::error_encoding::build_custom_error_message,
    compilation_context::ModuleId,
    data::DATA_ABORT_MESSAGE_PTR_OFFSET,
    translation::intermediate_types::{IntermediateType, structs::IStructType},
};

use super::{NativeFunction, error::NativeFunctionError};

/// Adds the native 'revert' function.
/// Expects the error type to be a struct. Each field of the error struct is loaded from memory and ABI-encoded to construct a revert reason message.
/// The encoding format follows the ABI convention for custom errors, as if calling a function named after the error type with its fields as parameters.
pub fn add_revert_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    error_itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_REVERT,
        compilation_ctx,
        &[error_itype],
        module_id,
    )?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    // Get the error type. Should be a struct, otherwise it panics.
    let error_struct = compilation_ctx
        .get_struct_by_intermediate_type(error_itype)
        .unwrap();

    let IStructType::AbiError = error_struct.type_ else {
        return Err(NativeFunctionError::RevertFunctionNoError(
            error_struct.identifier.clone(),
        ));
    };

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    // Arguments
    let error_struct_ptr = module.locals.add(ValType::I32);

    let encoded_error_ptr = build_custom_error_message(
        &mut builder,
        module,
        compilation_ctx,
        &error_struct,
        error_struct_ptr,
    )
    .map_err(|e| NativeFunctionError::Abi(Rc::new(e)))?;

    // Store the ptr at DATA_ABORT_MESSAGE_PTR_OFFSET
    builder
        .i32_const(DATA_ABORT_MESSAGE_PTR_OFFSET)
        .local_get(encoded_error_ptr)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // Return 1 to indicate an error occurred
    builder.i32_const(1);
    builder.return_();

    Ok(function.finish(vec![error_struct_ptr], &mut module.funcs))
}
