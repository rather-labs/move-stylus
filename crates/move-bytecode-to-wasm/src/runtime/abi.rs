use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg, UnaryOp},
};

use crate::{
    CompilationContext, data::RuntimeErrorData, error::RuntimeError,
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::RuntimeFunction;

/// Validates that a pointer value fits in 32 bits by checking that the first
/// 28 bytes are zero. This is used when reading ABI-encoded pointers to ensure they can
/// fit in WASM's 32-bit address space.
///
/// If any of the first 28 bytes are non-zero, the function will trap with an `Overflow` runtime error.
///
/// # WASM Function Arguments
/// * `pointer` (i32) - pointer to the memory location containing the 32-byte value to validate
/// * `memory_id` (implicit) - memory ID from compilation context
///
/// # WASM Function Returns
/// * Nothing (void)
pub fn validate_pointer_32_bit(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> FunctionId {
    let mut function_builder = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut function_body = function_builder.func_body();

    let pointer = module.locals.add(ValType::I32);

    // We are just assuming that the max value can fit in 32 bits, otherwise we cannot
    // reference WASM memory. If the value is greater than 32 bits, the function will return an Overflow runtime error.
    for i in 0..7 {
        function_body.block(None, |block| {
            let block_id = block.id();

            block
                .local_get(pointer)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: i * 4,
                    },
                )
                .unop(UnaryOp::I32Eqz)
                .br_if(block_id);

            block.return_error(
                module,
                compilation_ctx,
                None,
                runtime_error_data,
                RuntimeError::MemoryAccessOutOfBounds,
            );
        });
    }

    function_builder.name(RuntimeFunction::ValidatePointer32Bit.name().to_owned());
    function_builder.finish(vec![pointer], &mut module.funcs)
}
