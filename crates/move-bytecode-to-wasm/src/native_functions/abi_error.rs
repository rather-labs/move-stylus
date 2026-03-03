// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

use walrus::{FunctionBuilder, FunctionId, Module, ValType};

use crate::{
    CompilationContext,
    abi_types::error_encoding::build_custom_error_message,
    compilation_context::ModuleId,
    data::RuntimeErrorData,
    translation::intermediate_types::{IntermediateType, structs::IStructType},
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::{NativeFunction, error::NativeFunctionError};

/// Generates the native `revert` function for a concrete ABI error type.
///
/// The error type must be an ABI error struct. Each struct field is loaded from memory,
/// ABI-encoded as custom error payload, and forwarded to the runtime error handler so
/// execution terminates with the encoded revert data.
///
/// # WASM Function Arguments
/// * `error_struct_ptr` - (i32): pointer to the ABI error struct instance in memory
///
/// # WASM Function Returns
/// * None - execution reverts through runtime error handling
pub fn add_revert_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
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
    let error_struct = compilation_ctx.get_struct_by_intermediate_type(error_itype)?;

    let IStructType::AbiError = error_struct.type_ else {
        return Err(NativeFunctionError::RevertFunctionNoError(
            error_struct.identifier,
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
        runtime_error_data,
        &error_struct,
        error_struct_ptr,
    )?;

    builder
        .local_get(encoded_error_ptr)
        .add_handle_error_instructions(module, compilation_ctx, None);

    Ok(function.finish(vec![error_struct_ptr], &mut module.funcs))
}
