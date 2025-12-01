use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext, compilation_context::ModuleId,
    translation::intermediate_types::vector::IVector,
    wasm_builder_extensions::WasmBuilderExtension,
};

use super::NativeFunction;

/// Converts the raw calldata bytes into a vector<u8>
pub fn add_calldata_as_vector_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);

    let calldata_ptr = module.locals.add(ValType::I32);

    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_CALLDATA_AS_VECTOR,
            module_id,
        ))
        .func_body();

    // We need to convert the calldata to a vector<u8>, i.e. each byte takes 4 bytes (due to the current internal impl of vectors)
    let calldata_len = module.locals.add(ValType::I32);

    // Load the calldata length from the first 4 bytes
    builder
        .local_get(calldata_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(calldata_len);

    // Set the calldata pointer past the length
    builder
        .local_get(calldata_ptr)
        .i32_const(4)
        .binop(BinaryOp::I32Add)
        .local_set(calldata_ptr);

    let vector_ptr = module.locals.add(ValType::I32);
    IVector::allocate_vector_with_header(
        &mut builder,
        compilation_ctx,
        vector_ptr,
        calldata_len,
        calldata_len,
        4,
    );

    let i = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(i);
    builder.loop_(None, |loop_block| {
        let loop_block_id = loop_block.id();

        // address: vector_ptr + 8 (header) + i * 4
        loop_block.vec_elem_ptr(vector_ptr, i, 4);

        // value: calldata[i]
        loop_block
            .local_get(calldata_ptr)
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

        // continue the loop if i < calldata_len
        loop_block
            .local_get(i)
            .local_get(calldata_len)
            .binop(BinaryOp::I32LtU)
            .br_if(loop_block_id);
    });

    builder.local_get(vector_ptr);

    function.finish(vec![calldata_ptr], &mut module.funcs)
}

/// Returns the length of the calldata
pub fn add_calldata_length_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    module_id: &ModuleId,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let calldata_struct_ptr = module.locals.add(ValType::I32);

    let mut builder = function
        .name(NativeFunction::get_function_name(
            NativeFunction::NATIVE_CALLDATA_LENGTH,
            module_id,
        ))
        .func_body();

    // Load the length of the calldata from the first 4 bytes
    builder.local_get(calldata_struct_ptr).load(
        compilation_ctx.memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    function.finish(vec![calldata_struct_ptr], &mut module.funcs)
}
