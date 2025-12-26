use crate::{
    CompilationContext,
    runtime::{RuntimeFunction, RuntimeFunctionError},
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

pub fn pack_string_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let pack_u32_function = RuntimeFunction::PackU32.get(module, Some(compilation_ctx))?;

    let mut function_builder = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );
    let mut function_body = function_builder.func_body();

    let string_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);

    let data_pointer = module.locals.add(ValType::I32);
    let vector_pointer = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);
    let reference_value = module.locals.add(ValType::I32);
    let i = module.locals.add(ValType::I32);

    // String in move have the following form:
    // public struct String has copy, drop, store {
    //   bytes: vector<u8>,
    // }
    //
    // So we need to perform a load first to get to the inner vector
    function_body
        .local_get(string_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(vector_pointer);

    // Load the length
    function_body
        .local_get(vector_pointer)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    // Allocate space for the text, padding by 32 bytes plus 32 bytes for the length
    // Calculate: ((len + 31) & !31) + 32
    function_body
        .local_get(len)
        .i32_const(31)
        .binop(BinaryOp::I32Add)
        .i32_const(!31)
        .binop(BinaryOp::I32And)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .call(compilation_ctx.allocator)
        .local_set(data_pointer);

    // The value stored at this param position should be the distance from the start of this
    // calldata portion to the pointer
    function_body
        .local_get(data_pointer)
        .local_get(calldata_reference_pointer)
        .binop(BinaryOp::I32Sub)
        .local_set(reference_value);

    // Write the offset at writer_pointer
    function_body
        .local_get(reference_value)
        .local_get(writer_pointer)
        .call(pack_u32_function);

    // Set the vector pointer to point to the first element (skip vector header)
    function_body
        .local_get(vector_pointer)
        .i32_const(8)
        .binop(BinaryOp::I32Add)
        .local_set(vector_pointer);

    // Write the length at data_pointer
    function_body
        .local_get(len)
        .local_get(data_pointer)
        .call(pack_u32_function);

    // Increment the data pointer to point to the data area
    function_body
        .local_get(data_pointer)
        .i32_const(32)
        .binop(BinaryOp::I32Add)
        .local_set(data_pointer);

    // Outer block: if the vector length is 0, we skip to the end
    function_body.block(None, |outer_block| {
        let outer_block_id = outer_block.id();

        // Check if length == 0
        outer_block
            .local_get(len)
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(outer_block_id);

        // Loop through the vector values
        outer_block.i32_const(0).local_set(i);
        outer_block.loop_(None, |loop_block| {
            let loop_block_id = loop_block.id();

            // Load byte from vector and store at data_pointer
            loop_block
                .local_get(data_pointer)
                .local_get(vector_pointer)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32_8 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            // Increment the vector pointer by 1 byte
            loop_block
                .local_get(vector_pointer)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_set(vector_pointer);

            // Increment the data pointer by 1 byte
            loop_block
                .local_get(data_pointer)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_set(data_pointer);

            // Increment i
            loop_block
                .local_get(i)
                .i32_const(1)
                .binop(BinaryOp::I32Add)
                .local_tee(i);

            // Continue loop if i < len
            loop_block
                .local_get(len)
                .binop(BinaryOp::I32LtU)
                .br_if(loop_block_id);
        });
    });

    function_builder.name(RuntimeFunction::PackString.name().to_owned());
    Ok(function_builder.finish(
        vec![string_pointer, writer_pointer, calldata_reference_pointer],
        &mut module.funcs,
    ))
}
