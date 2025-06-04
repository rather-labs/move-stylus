use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg},
};

use super::RuntimeFunction;

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

                // Read both numbers bytes at offset and compare them
                loop_
                    .local_get(a_ptr)
                    .local_get(offset)
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
                    )
                    .local_get(b_ptr)
                    .local_get(offset)
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
                    .i32_const(1)
                    .binop(BinaryOp::I32Add)
                    .local_set(offset)
                    .br(loop_id);
            });
        })
        // If we get here, we looped both structures and all the bytes were equal
        .i32_const(1);

    function.finish(vec![a_ptr, b_ptr, type_heap_size], &mut module.funcs)
}
