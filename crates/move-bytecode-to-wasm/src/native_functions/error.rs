use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    abi_types::error_encoding::{build_custom_error_message, move_signature_to_abi_selector},
    abi_types::packing::build_pack_instructions,
    compilation_context::ModuleId,
    data::DATA_ABORT_MESSAGE_PTR_OFFSET,
    translation::intermediate_types::IntermediateType,
};

use super::NativeFunction;

/// Adds thenative 'revert' function that
pub fn add_revert_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    error_itype: &IntermediateType,
    module_id: &ModuleId,
) -> FunctionId {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_REVERT,
        compilation_ctx,
        &[error_itype],
        module_id,
    );
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    // Get the error type. Should be a struct, otherwise it panics.
    let error_struct = compilation_ctx
        .get_struct_by_intermediate_type(error_itype)
        .unwrap();

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    // Arguments
    let error_struct_ptr = module.locals.add(ValType::I32);

    // Load each field of the error struct for ABI encoding them.
    for (index, field) in error_struct.fields.iter().enumerate() {
        // Load each field's middle pointer
        builder.local_get(error_struct_ptr).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );

        // If the field is a stack type, load the value from memory
        if field.is_stack_type() {
            if field.stack_data_size() == 8 {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            } else {
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
        }
    }

    // Pack the error data as if it was for a call to a function with the error struct fields as arguments
    let (error_data_ptr, error_data_len) =
        build_pack_instructions(&mut builder, &error_struct.fields, module, compilation_ctx);

    // Calculate the error selector
    let error_selector = move_signature_to_abi_selector(
        &error_struct.identifier,
        &error_struct.fields,
        compilation_ctx,
    );

    // Build the abi encoded error message
    let encoded_error_ptr = build_custom_error_message(
        &mut builder,
        module,
        compilation_ctx,
        &error_selector,
        error_data_ptr,
        error_data_len,
    );

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

    function.finish(vec![error_struct_ptr], &mut module.funcs)
}
