use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext, compilation_context::ModuleId,
    native_functions::error::NativeFunctionError,
    translation::intermediate_types::IntermediateType,
    translation::intermediate_types::vector::IVector,
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::NativeFunction;

/// Converts the raw bytes<M> into a vector<u8>
pub fn add_as_vector_bytes_n_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
    module_id: &ModuleId,
) -> Result<FunctionId, NativeFunctionError> {
    let name = NativeFunction::get_generic_function_name(
        NativeFunction::NATIVE_AS_VEC_BYTES_N,
        compilation_ctx,
        &[&itype.clone()],
        module_id,
    )?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    let bytes_ptr = module.locals.add(ValType::I32);
    let bytes_n = module.locals.add(ValType::I32);

    let vector_ptr = module.locals.add(ValType::I32);
    IVector::allocate_vector_with_header(
        &mut builder,
        compilation_ctx,
        vector_ptr,
        bytes_n,
        bytes_n,
        4,
    );

    let i = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(i);
    builder.loop_(None, |loop_block| {
        let loop_block_id = loop_block.id();

        // address: vector_ptr + 8 (header) + i * 4
        loop_block.vec_elem_ptr(vector_ptr, i, 4);

        // value: bytesN[i]
        loop_block
            .local_get(bytes_ptr)
            .local_get(i)
            .binop(BinaryOp::I32Add)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32_8 {
                    kind: ExtendedLoad::ZeroExtend,
                },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        // Store the i-th value at the i-th position of the vector
        loop_block.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // increment i
        loop_block
            .local_get(i)
            .i32_const(1)
            .binop(BinaryOp::I32Add)
            .local_set(i);

        // continue the loop if i < bytes_n
        loop_block
            .local_get(i)
            .local_get(bytes_n)
            .binop(BinaryOp::I32LtU)
            .br_if(loop_block_id);
    });

    builder.local_get(vector_ptr);

    Ok(function.finish(vec![bytes_ptr, bytes_n], &mut module.funcs))
}
