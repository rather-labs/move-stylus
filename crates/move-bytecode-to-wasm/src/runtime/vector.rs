use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use super::RuntimeFunction;
use crate::CompilationContext;
use crate::wasm_builder_extensions::WasmBuilderExtension;

// Increments vector length by 1
// # Arguments:
//    - len: (i32) length of the vector
//    - vec ptr: (i32) reference to the vector
pub fn increment_vec_len_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::VecIncrementLen.name().to_owned())
        .func_body();

    let ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    builder
        .local_get(ptr)
        .local_get(len)
        .i32_const(1)
        .binop(BinaryOp::I32Add)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    function.finish(vec![ptr, len], &mut module.funcs)
}

// Decrements vector length by 1
// # Arguments:
//    - len: (i32) length of the vector
//    - vec ptr: (i32) reference to the vector
pub fn decrement_vec_len_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::VecDecrementLen.name().to_owned())
        .func_body();

    let ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    // Trap if vector length == 0
    builder
        .local_get(len)
        .i32_const(0)
        .binop(BinaryOp::I32Eq)
        .if_else(
            None,
            |then| {
                then.unreachable(); // cannot pop from empty vector
            },
            |_| {},
        );

    builder
        .local_get(ptr)
        .local_get(len)
        .i32_const(1)
        .binop(BinaryOp::I32Sub)
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    function.finish(vec![ptr, len], &mut module.funcs)
}

/// Swaps the elements at two indices in the vector. Abort the execution if any of the indice
/// is out of bounds.
///
/// ```..., vector_reference, u64_value(1), u64_value(2) -> ...```
pub fn vec_swap_32_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I64, ValType::I64],
        &[],
    );
    let mut builder = function
        .name(RuntimeFunction::VecSwap32.name().to_owned())
        .func_body();

    let ptr = module.locals.add(ValType::I32);
    let idx1_i64 = module.locals.add(ValType::I64);
    let idx2_i64 = module.locals.add(ValType::I64);

    let idx2 = module.locals.add(ValType::I32);
    let idx1 = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    let downcast_f = RuntimeFunction::DowncastU64ToU32.get(module, None);

    builder.local_get(idx1_i64).call(downcast_f).local_set(idx1);
    builder.local_get(idx2_i64).call(downcast_f).local_set(idx2);

    // Load vector ptr and len
    builder
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    builder.block(None, |block| {
        let block_id = block.id();

        block
            .local_get(idx1_i64)
            .local_get(idx2_i64)
            .binop(BinaryOp::I64Eq)
            .br_if(block_id);

        // Helper: emit trap if idx >= len
        let trap_if_idx_oob = |b: &mut InstrSeqBuilder, idx: LocalId| {
            b.local_get(idx)
                .local_get(len)
                .binop(BinaryOp::I32GeU)
                .if_else(
                    None,
                    |then_| {
                        then_.unreachable();
                    },
                    |_| {},
                );
        };

        trap_if_idx_oob(block, idx1);
        trap_if_idx_oob(block, idx2);

        // Swap elements
        let aux = module.locals.add(ValType::I32);

        let ptr1 = module.locals.add(ValType::I32);
        let ptr2 = module.locals.add(ValType::I32);

        block.vec_elem_ptr(ptr, idx1, 4);
        block.local_set(ptr1);

        block.vec_elem_ptr(ptr, idx2, 4);
        block.local_set(ptr2);

        // Load elem 1 into aux
        block
            .local_get(ptr1)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(aux);

        // Store elem 2 into ptr1
        block
            .local_get(ptr1)
            .local_get(ptr2)
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
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        // Store elem 1 into ptr2
        block.local_get(ptr2).local_get(aux).store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );
    });
    function.finish(vec![ptr, idx1_i64, idx2_i64], &mut module.funcs)
}

/// Swaps the elements at two indices in the vector. Abort the execution if any of the indice
/// is out of bounds.
///
/// ```..., vector_reference, u64_value(1), u64_value(2) -> ...```
pub fn vec_swap_64_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I64, ValType::I64],
        &[],
    );
    let mut builder = function
        .name(RuntimeFunction::VecSwap64.name().to_owned())
        .func_body();

    let ptr = module.locals.add(ValType::I32);
    let idx1_i64 = module.locals.add(ValType::I64);
    let idx2_i64 = module.locals.add(ValType::I64);

    let idx2 = module.locals.add(ValType::I32);
    let idx1 = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    let downcast_f = RuntimeFunction::DowncastU64ToU32.get(module, None);

    builder.local_get(idx1_i64).call(downcast_f).local_set(idx1);
    builder.local_get(idx2_i64).call(downcast_f).local_set(idx2);

    // Load vector ptr and len
    builder
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    builder.block(None, |block| {
        let block_id = block.id();

        block
            .local_get(idx1_i64)
            .local_get(idx2_i64)
            .binop(BinaryOp::I64Eq)
            .br_if(block_id);

        // Helper: emit trap if idx >= len
        let trap_if_idx_oob = |b: &mut InstrSeqBuilder, idx: LocalId| {
            b.local_get(idx)
                .local_get(len)
                .binop(BinaryOp::I32GeU)
                .if_else(
                    None,
                    |then_| {
                        then_.unreachable();
                    },
                    |_| {},
                );
        };

        trap_if_idx_oob(block, idx1);
        trap_if_idx_oob(block, idx2);

        // Swap elements
        let aux = module.locals.add(ValType::I64);

        let ptr1 = module.locals.add(ValType::I32);
        let ptr2 = module.locals.add(ValType::I32);

        block.vec_elem_ptr(ptr, idx1, 8);
        block.local_set(ptr1);

        block.vec_elem_ptr(ptr, idx2, 8);
        block.local_set(ptr2);

        // Load elem 1 into aux
        block
            .local_get(ptr1)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(aux);

        // Store elem 2 into ptr1
        block
            .local_get(ptr1)
            .local_get(ptr2)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        // Store elem 1 into ptr2
        block.local_get(ptr2).local_get(aux).store(
            compilation_ctx.memory_id,
            StoreKind::I64 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );
    });
    function.finish(vec![ptr, idx1_i64, idx2_i64], &mut module.funcs)
}

/// Pop an element from the end of vector. Aborts if the vector is empty.
///
/// Stack transition:
///
/// ```..., vector_reference -> ..., element```
pub fn vec_pop_back_32_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::VecPopBack32.name().to_owned())
        .func_body();

    let size = 4;
    let ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    builder
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    // Decrement vector length
    builder
        .local_get(ptr)
        .local_get(len)
        .call(RuntimeFunction::VecDecrementLen.get(module, Some(compilation_ctx)));

    // Update vector length
    builder
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    builder.vec_elem_ptr(ptr, len, size);

    builder.load(
        compilation_ctx.memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    function.finish(vec![ptr], &mut module.funcs)
}

/// Pop an element from the end of vector. Aborts if the vector is empty.
///
/// Stack transition:
///
/// ```..., vector_reference -> ..., element```
pub fn vec_pop_back_64_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I64]);
    let mut builder = function
        .name(RuntimeFunction::VecPopBack64.name().to_owned())
        .func_body();

    let size = 8;
    let ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    builder
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    // Decrement vector length
    builder
        .local_get(ptr)
        .local_get(len)
        .call(RuntimeFunction::VecDecrementLen.get(module, Some(compilation_ctx)));

    // Update vector length
    builder
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    builder.vec_elem_ptr(ptr, len, size);

    builder.load(
        compilation_ctx.memory_id,
        LoadKind::I64 { atomic: false },
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    function.finish(vec![ptr], &mut module.funcs)
}

/// Pushes a pointer to a non-heap element in a vector.
///
/// # Arguments:
///    - vector_reference: (i32) reference to the vector
///    - index: (i64) index of the element to borrow
///    - is_heap: (i32) boolean indicating if the element is heap or not
///    - size: (i32) stack size of the vector inner type
/// # Returns:
///    - i32 reference to the borrowed element
pub fn vec_borrow_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::VecBorrow.name().to_owned())
        .func_body();

    // Local variables
    let is_heap = module.locals.add(ValType::I32);
    let size = module.locals.add(ValType::I32);
    let index = module.locals.add(ValType::I32);
    let vec_ref = module.locals.add(ValType::I32);
    let vec_ptr = module.locals.add(ValType::I32);

    // Load vector reference
    builder
        .local_get(vec_ref)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(vec_ptr);

    // Trap if index >= length
    builder.block(None, |block| {
        block
            .local_get(vec_ptr)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_get(index)
            .binop(BinaryOp::I32GtU);
        block.br_if(block.id());
        block.unreachable();
    });

    // If the element is stored on the heap, we directly return vec_elem_ptr, as it is already a reference (pointer to a pointer).
    // If the element is not on the heap, we convert the pointer returned by vec_elem_ptr into a reference by wrapping it.
    builder.local_get(is_heap).if_else(
        ValType::I32,
        |then| {
            then.vec_elem_ptr_dynamic(vec_ptr, index, size);
        },
        |else_| {
            let elem_ref = module.locals.add(ValType::I32);
            else_
                .i32_const(4)
                .call(compilation_ctx.allocator)
                .local_tee(elem_ref)
                .vec_elem_ptr_dynamic(vec_ptr, index, size)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_get(elem_ref);
        },
    );

    function.finish(vec![vec_ref, index, is_heap, size], &mut module.funcs)
}
