use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

use super::RuntimeFunction;
use crate::wasm_builder_extensions::WasmBuilderExtension;

/// Verifies if two elements a and b are equal
///
/// # Arguments
///    - pointer to a
///    - pointer to b
///    - How many bytes occupies in memory
/// # Returns:
///    - a == b
pub fn a_equals_b(module: &mut Module, compilation_ctx: &crate::CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    // Function arguments
    let a_ptr = module.locals.add(ValType::I32);
    let b_ptr = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Local variables
    let offset = module.locals.add(ValType::I32);

    let mut builder = function
        .name(RuntimeFunction::HeapTypeEquality.name().to_owned())
        .func_body();

    builder
        .block(None, |block| {
            let block_id = block.id();

            // If a_ptr == b_ptr we return true
            block
                .local_get(a_ptr)
                .local_get(b_ptr)
                .binop(BinaryOp::I32Eq)
                .br_if(block_id);

            block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                // If we finished processing, we exit
                loop_
                    .local_get(offset)
                    .local_get(type_heap_size)
                    .binop(BinaryOp::I32Eq)
                    .br_if(block_id);

                // Read both pointers at offset and compare them
                loop_
                    .local_get(a_ptr)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_get(b_ptr)
                    .local_get(offset)
                    .binop(BinaryOp::I32Add)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    // If we find that some chunk is not equal, we exit with false
                    .binop(BinaryOp::I32Ne)
                    .if_else(
                        None,
                        |then| {
                            then.i32_const(0).return_();
                        },
                        |_| {},
                    );

                loop_
                    .local_get(offset)
                    .i32_const(4)
                    .binop(BinaryOp::I32Add)
                    .local_set(offset)
                    .br(loop_id);
            });
        })
        // If we get here, we looped both structures and all the bytes were equal
        .i32_const(1);

    function.finish(vec![a_ptr, b_ptr, type_heap_size], &mut module.funcs)
}

/// Receives two vectors pointers that contain heap types, loop over all the pointers and compare
/// by equality each pair of data
///
/// # Arguments
///    - pointer to v1
///    - pointer to v2
///    - length of both vectors (must be the same)
///    - How many bytes the inner vector type occupies in memory
/// # Returns:
///    - v1 == v2
pub fn vec_equality_heap_type(
    module: &mut Module,
    compilation_ctx: &crate::CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );

    let mut builder = function
        .name(RuntimeFunction::VecEqualityHeapType.name().to_owned())
        .func_body();

    let equality_f_id = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));

    // Function arguments
    let v1_ptr = module.locals.add(ValType::I32);
    let v2_ptr = module.locals.add(ValType::I32);
    let length = module.locals.add(ValType::I32);
    let type_heap_size = module.locals.add(ValType::I32);

    // Local variables
    let res = module.locals.add(ValType::I32);
    let offset = module.locals.add(ValType::I32);
    let size = module.locals.add(ValType::I32);

    // Set res to true and offset to 0
    builder
        .i32_const(1)
        .local_set(res)
        .i32_const(0)
        .local_set(offset);

    // Skip the length and capacity of the vectors
    builder.skip_vec_header(v1_ptr).local_set(v1_ptr);
    builder.skip_vec_header(v2_ptr).local_set(v2_ptr);

    // Set the size as the length * 4 (pointer size)
    builder
        .local_get(length)
        .i32_const(4)
        .binop(BinaryOp::I32Mul)
        .local_set(size);

    // We must follow pointer by pointer and use the equality function
    builder.block(None, |block| {
        let block_id = block.id();

        block.loop_(None, |loop_| {
            let loop_id = loop_.id();

            // If we are at the end of the loop means we finished comparing,
            // so we break the loop with the true in res
            loop_
                .local_get(size)
                .local_get(offset)
                .binop(BinaryOp::I32Eq)
                .br_if(block_id);

            // Load both pointers into stack
            loop_
                .local_get(v1_ptr)
                .local_get(offset)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_get(v2_ptr)
                .local_get(offset)
                .binop(BinaryOp::I32Add)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            loop_
                .local_get(type_heap_size)
                .call(equality_f_id)
                // If they are equal we continue the loop
                // Otherwise, we leave set res as false and break the loop
                .if_else(
                    None,
                    |then| {
                        then.local_get(offset)
                            .i32_const(4)
                            .binop(BinaryOp::I32Add)
                            .local_set(offset)
                            .br(loop_id);
                    },
                    |else_| {
                        else_.i32_const(0).local_set(res).br(block_id);
                    },
                );
        });
    });

    builder.local_get(res);

    function.finish(
        vec![v1_ptr, v2_ptr, length, type_heap_size],
        &mut module.funcs,
    )
}
