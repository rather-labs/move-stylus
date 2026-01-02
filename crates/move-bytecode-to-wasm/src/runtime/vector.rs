use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use super::{RuntimeFunction, error::RuntimeFunctionError};
use crate::CompilationContext;
use crate::translation::intermediate_types::{
    IntermediateType, error::IntermediateTypeError, heap_integers::IU128,
};
use crate::wasm_builder_extensions::WasmBuilderExtension;

/// Increments vector length by 1
///
/// # WASM Function Arguments
/// * `vec_ptr` (i32) - reference to the vector
pub fn increment_vec_len_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::VecIncrementLen.name().to_owned())
        .func_body();

    let ptr = module.locals.add(ValType::I32);

    builder
        .local_get(ptr)
        .local_get(ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
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

    function.finish(vec![ptr], &mut module.funcs)
}

/// Decrements vector length by 1
///
/// # WASM Function Arguments
/// * `ptr` (i32) - pointer to the vector
pub fn decrement_vec_len_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::VecDecrementLen.name().to_owned())
        .func_body();

    let ptr = module.locals.add(ValType::I32);

    // Trap if vector length == 0
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
        .unop(UnaryOp::I32Eqz)
        .if_else(
            None,
            |then| {
                then.unreachable(); // cannot pop from empty vector
            },
            |else_| {
                else_
                    .local_get(ptr)
                    .local_get(ptr)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
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
            },
        );

    function.finish(vec![ptr], &mut module.funcs)
}

/// Swaps the elements at two indices in the vector. Abort the execution if any of the indices
/// is out of bounds.
///
/// Stack transition:
/// ```..., vector_reference, u64_value(1), u64_value(2) -> ...```
///
/// # WASM Function Arguments
/// * `ptr` (i32) - pointer to the vector
/// * `idx1_i64` (i64) - first index
/// * `idx2_i64` (i64) - second index
///
/// # Arguments
/// * `inner_type` - The intermediate type of the vector's inner elements
pub fn vec_swap_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner_type: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let element_size = inner_type.wasm_memory_data_size()?;
    let load_kind = inner_type.load_kind()?;
    let store_kind = inner_type.store_kind()?;

    let elem_val_type = ValType::try_from(inner_type)?;

    let name =
        RuntimeFunction::VecSwap.get_generic_function_name(compilation_ctx, &[inner_type])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I64, ValType::I64],
        &[],
    );

    let mut builder = function.name(name).func_body();

    // Arguments
    let vector_ref = module.locals.add(ValType::I32);
    let idx1_i64 = module.locals.add(ValType::I64);
    let idx2_i64 = module.locals.add(ValType::I64);

    let idx2 = module.locals.add(ValType::I32);
    let idx1 = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    let downcast_f = RuntimeFunction::DowncastU64ToU32.get(module, None)?;

    builder.local_get(idx1_i64).call(downcast_f).local_set(idx1);
    builder.local_get(idx2_i64).call(downcast_f).local_set(idx2);

    // Load vector ptr and len
    builder
        .local_get(vector_ref)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(vector_ref)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    let vector_ptr = vector_ref;

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
        let aux = module.locals.add(elem_val_type);

        let ptr1 = module.locals.add(ValType::I32);
        let ptr2 = module.locals.add(ValType::I32);

        block.vec_elem_ptr(vector_ptr, idx1, element_size);
        block.local_set(ptr1);

        block.vec_elem_ptr(vector_ptr, idx2, element_size);
        block.local_set(ptr2);

        // Load elem 1 into aux
        block
            .local_get(ptr1)
            .load(
                compilation_ctx.memory_id,
                load_kind,
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
                load_kind,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .store(
                compilation_ctx.memory_id,
                store_kind,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        // Store elem 1 into ptr2
        block.local_get(ptr2).local_get(aux).store(
            compilation_ctx.memory_id,
            store_kind,
            MemArg {
                align: 0,
                offset: 0,
            },
        );
    });

    Ok(function.finish(vec![vector_ref, idx1_i64, idx2_i64], &mut module.funcs))
}

/// Pop an element from the end of vector. Aborts if the vector is empty.
///
/// Stack transition:
/// ```..., vector_reference -> ..., element```
///
/// # WASM Function Arguments
/// * `vector_ref` (i32) - reference to the vector
///
/// # WASM Function Returns
/// * element - type depends on `inner_type`
///
/// # Arguments
/// * `inner_type` - The intermediate type of the vector's inner elements
pub fn vec_pop_back_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner_type: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let element_size = inner_type.wasm_memory_data_size()?;
    let load_kind = inner_type.load_kind()?;
    let return_type = ValType::try_from(inner_type)?;

    let name =
        RuntimeFunction::VecPopBack.get_generic_function_name(compilation_ctx, &[inner_type])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let decrement_vec_len_function =
        RuntimeFunction::VecDecrementLen.get(module, Some(compilation_ctx))?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[return_type]);
    let mut builder = function.name(name).func_body();

    let vector_ref = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    builder
        .local_get(vector_ref)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(vector_ref)
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
        .local_get(vector_ref)
        .call(decrement_vec_len_function);

    // Update vector length
    builder
        .local_get(vector_ref)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    builder.vec_elem_ptr(vector_ref, len, element_size);

    builder.load(
        compilation_ctx.memory_id,
        load_kind,
        MemArg {
            align: 0,
            offset: 0,
        },
    );

    Ok(function.finish(vec![vector_ref], &mut module.funcs))
}

/// Appends an element to the end of a vector.
/// If the vector's capacity is greater than its length, the element is simply added at the next available position.
/// If the vector's capacity equals its length, a new vector is created with double the current length as its capacity,
/// the existing elements are copied into this new vector, and then the element is pushed.
///
/// # Stack Arguments
///
/// * `elem`: (i32/i64) The element to be pushed.
/// * `vec_ref`: (i32) A reference to the vector.
pub fn vec_push_back_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::VecPushBack.get_generic_function_name(compilation_ctx, &[inner])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let inner_valtype = ValType::try_from(inner)?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, inner_valtype], &[]);
    let mut builder = function.name(name).func_body();

    // Arguments
    let vec_ref = module.locals.add(ValType::I32);
    let elem = module.locals.add(inner_valtype);

    // Locals
    let vec_ptr = module.locals.add(ValType::I32);
    let len = module.locals.add(ValType::I32);

    let size = inner.wasm_memory_data_size()?;

    // Load and set the vector pointer
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
        .local_tee(vec_ptr);

    // Load and set the vector length
    builder
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(len);

    // Load the vector capacity
    builder.local_get(vec_ptr).load(
        compilation_ctx.memory_id,
        LoadKind::I32 { atomic: false },
        MemArg {
            align: 0,
            offset: 4,
        },
    );

    // Check if len == capacity. If true, we copy the original vector but doubling its capacity.
    let copy_local_function =
        RuntimeFunction::VecCopyLocal.get_generic(module, compilation_ctx, &[inner])?;
    builder.binop(BinaryOp::I32Eq).if_else(
        None,
        |then| {
            // Copy the vector but doubling its capacity
            then.local_get(vec_ptr)
                .i32_const(2)
                .call(copy_local_function);

            // Set vec_ptr to the new vector pointer and store it at *vec_ref
            // This modifies the original vector reference to point to the new vector
            let new_vec_ptr = module.locals.add(ValType::I32);
            then.local_set(new_vec_ptr)
                .local_get(vec_ref)
                .local_get(new_vec_ptr)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

            // Mark the original vector location with the DEADBEEF flag to indicate relocation.
            // When a vector is resized, any existing mutable references pointing to the old
            // location become invalid. By writing DEADBEEF into the length field (first 4 bytes)
            // of the original vector, we create a marker that can be detected after function calls.
            // After a call_indirect, we check for this flag and update any mutable references
            // that still point to the old location, following the chain to the new vector.
            then.local_get(vec_ptr)
                .i32_const(0xDEADBEEF_u32 as i32)
                .store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            // Set the original vector pointer to the new vector pointer.
            // This way we can update the reference to it, as explained above
            then.local_get(vec_ptr).local_get(new_vec_ptr).store(
                compilation_ctx.memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 4,
                },
            );
            then.local_get(new_vec_ptr).local_set(vec_ptr);
        },
        |_| {},
    );

    // Store the element in the next free position
    builder
        .vec_elem_ptr(vec_ptr, len, size)
        .local_get(elem)
        .store(
            compilation_ctx.memory_id,
            inner.store_kind()?,
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    // length++
    let vec_increment_len_fn =
        RuntimeFunction::VecIncrementLen.get(module, Some(compilation_ctx))?;
    builder.local_get(vec_ptr).call(vec_increment_len_fn);

    Ok(function.finish(vec![vec_ref, elem], &mut module.funcs))
}
/// Pushes a pointer to a non-heap element in a vector.
///
/// # WASM Function Arguments
/// * `vector_reference` - (i32) reference to the vector
/// * `index` - (i64) index of the element to borrow
/// * `is_heap` - (i32) boolean indicating if the element is heap or not
/// * `size` - (i32) stack size of the vector inner type
///
/// # WASM Function Returns
/// * i32 reference to the borrowed element
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

// Follows a chain of relocated vectors by checking for the DEADBEEF flag.
//
// When a vector's capacity is increased (e.g., during push_back), the original vector data
// is copied to a new memory location with increased capacity. This invalidates any references
// pointing to the original vector location.
//
// To handle this, when relocating a vector, we write a special marker (DEADBEEF) into the
// first 4 bytes of the original vector's memory (where the length field was), and store the
// new vector pointer at offset 4 (where the capacity field was). This is safe because a
// vector's metadata requires at least 8 bytes: 4 bytes for length and 4 bytes for capacity.
//
// This function checks if the vector pointer points to a location containing the DEADBEEF
// flag. If so, it follows the chain by reading the new pointer from offset 4 and updating
// the reference. This process repeats until a valid vector (without the DEADBEEF flag) is found.
//
/// # WASM Function Arguments
/// * `vec_ref_ptr` - (i32) pointer to the reference structure that contains the vector pointer
pub fn vec_update_mut_ref_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::VecUpdateMutRef.name().to_owned())
        .func_body();

    let vec_ref_ptr = module.locals.add(ValType::I32);

    builder.loop_(None, |loop_block| {
        let loop_id = loop_block.id();

        // Check if DEADBEEF flag is set at *vec_ref
        loop_block
            .local_get(vec_ref_ptr)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .i32_const(0xDEADBEEF_u32 as i32)
            .binop(BinaryOp::I32Eq)
            .if_else(
                None,
                |then_| {
                    // If true, update the reference to point to the new vector,
                    // which is stored at *vec_ref + 4, and continue looping.
                    then_
                        .local_get(vec_ref_ptr)
                        .local_get(vec_ref_ptr)
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 4,
                            },
                        )
                        .store(
                            compilation_ctx.memory_id,
                            StoreKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        )
                        .br(loop_id);
                },
                |_| {
                    // If false, fall through and exit the loop.
                },
            );
    });

    function.finish(vec![vec_ref_ptr], &mut module.funcs)
}

/// Allocates memory for a vector with a header of 8 bytes.
/// First 4 bytes are the length, next 4 bytes are the capacity.
pub fn allocate_vector_with_header_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::AllocateVectorWithHeader.name().to_owned())
        .func_body();

    // Arguments
    let len = module.locals.add(ValType::I32);
    let capacity = module.locals.add(ValType::I32);
    let data_size = module.locals.add(ValType::I32);

    // Local pointer to the allocated memory
    let pointer = module.locals.add(ValType::I32);

    // If the len is 0 we just allocate 8 bytes representing 0 length and 0 capacity
    builder
        .local_get(capacity)
        .i32_const(0)
        .binop(BinaryOp::I32Eq)
        .if_else(
            None,
            |then| {
                then.i32_const(8)
                    .call(compilation_ctx.allocator)
                    .local_set(pointer);
            },
            |else_| {
                // This is a failsafe to prevent UB if static checks failed
                else_
                    .local_get(len)
                    .local_get(capacity)
                    .binop(BinaryOp::I32GtU)
                    .if_else(
                        None,
                        |then_| {
                            then_.unreachable(); // Trap if len > capacity
                        },
                        |_| {},
                    );

                // Allocate memory: capacity * element size + 8 bytes for header
                else_
                    .local_get(capacity)
                    .local_get(data_size)
                    .binop(BinaryOp::I32Mul)
                    .i32_const(8)
                    .binop(BinaryOp::I32Add)
                    .call(compilation_ctx.allocator)
                    .local_set(pointer);

                // Write length at offset 0
                else_.local_get(pointer).local_get(len).store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                // Write capacity at offset 4
                else_.local_get(pointer).local_get(capacity).store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 4,
                    },
                );
            },
        );

    // Return the pointer
    builder.local_get(pointer);

    function.finish(vec![len, capacity, data_size], &mut module.funcs)
}

/// Perform a deep copy of a vector.
///
/// # Stack Arguments
///
/// * `multiplier`: (i32) A factor used to determine the new vector's capacity, calculated as `multiplier * len`.
/// * `src_ptr`: (i32) A pointer referencing the vector to be duplicated.
///
/// # Returns
///
/// * `dst_ptr`: (i32) A pointer to the newly copied vector.
pub fn copy_local_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name =
        RuntimeFunction::VecCopyLocal.get_generic_function_name(compilation_ctx, &[inner])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    // Arguments
    let src_ptr = module.locals.add(ValType::I32);
    let multiplier = module.locals.add(ValType::I32);

    // === Local declarations ===
    let dst_ptr = module.locals.add(ValType::I32); // pointer to the newly copied vector
    let index = module.locals.add(ValType::I32); // index of the current element being copied
    let len = module.locals.add(ValType::I32); // length of the original vector
    let capacity = module.locals.add(ValType::I32); // capacity of the new vector
    let data_size = inner.wasm_memory_data_size()?; // size of the inner type data in the vector

    // === Set vector ptr and length ===
    builder
        .local_get(src_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_set(len);

    // Calculate the capacity
    builder
        .local_get(len)
        .i32_const(0)
        .binop(BinaryOp::I32Eq)
        .if_else(
            None,
            |then| {
                then.i32_const(1).local_set(capacity);
            },
            |else_| {
                else_
                    .local_get(len)
                    .local_get(multiplier)
                    .binop(BinaryOp::I32Mul)
                    .local_set(capacity);
            },
        );

    // Allocate memory and write length and capacity at the beginning
    let allocate_vector_with_header_function =
        RuntimeFunction::AllocateVectorWithHeader.get(module, Some(compilation_ctx))?;
    builder
        .local_get(len)
        .local_get(capacity)
        .i32_const(data_size)
        .call(allocate_vector_with_header_function)
        .local_set(dst_ptr);

    // === Loop  ===
    builder.i32_const(0).local_set(index);

    // Aux locals for the loop
    let src_elem_ptr = module.locals.add(ValType::I32);
    let dst_elem_ptr = module.locals.add(ValType::I32);

    // Get nested vector copy function if needed (before entering closure)
    let nested_copy_function = if let IntermediateType::IVector(inner_) = inner {
        Some(copy_local_function(module, compilation_ctx, inner_)?)
    } else {
        None
    };

    // Outer block: if the vector length is 0, we skip to the end
    let mut inner_result = Ok(());
    builder.block(None, |outer_block| {
        let outer_block_id = outer_block.id();

        // Check if length == 0
        outer_block
            .local_get(len)
            .i32_const(0)
            .binop(BinaryOp::I32Eq)
            .br_if(outer_block_id);

        outer_block.loop_(None, |loop_block| {
            loop_block.vec_elem_ptr(dst_ptr, index, data_size); // where to store the element
            loop_block.vec_elem_ptr(src_ptr, index, data_size); // where to read the element

            inner_result = (|| {
                match inner {
                    IntermediateType::IBool
                    | IntermediateType::IU8
                    | IntermediateType::IU16
                    | IntermediateType::IU32
                    | IntermediateType::IU64 => {
                        loop_block.load(
                            compilation_ctx.memory_id,
                            inner.load_kind()?,
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );
                    }
                    IntermediateType::IU128 => {
                        // Set src
                        loop_block
                            .load(
                                compilation_ctx.memory_id,
                                LoadKind::I32 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            .local_set(src_elem_ptr);

                        // Allocate memory for dest
                        loop_block
                            .i32_const(16)
                            .call(compilation_ctx.allocator)
                            .local_tee(dst_elem_ptr);

                        // Put dest (tee above), src and size to perform memory copy
                        loop_block
                            .local_get(src_elem_ptr)
                            .i32_const(IU128::HEAP_SIZE);

                        loop_block
                            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                        loop_block.local_get(dst_elem_ptr);
                    }
                    IntermediateType::IU256 | IntermediateType::IAddress => {
                        loop_block
                            .load(
                                compilation_ctx.memory_id,
                                LoadKind::I32 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset: 0,
                                },
                            )
                            .local_set(src_elem_ptr);

                        loop_block
                            .i32_const(32)
                            .call(compilation_ctx.allocator)
                            .local_tee(dst_elem_ptr);

                        // Put dest (tee above), src and size to perform memory copy
                        loop_block.local_get(src_elem_ptr).i32_const(32);

                        loop_block
                            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

                        loop_block.local_get(dst_elem_ptr);
                    }
                    IntermediateType::IVector(_) => {
                        // Source pointer
                        loop_block.load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );

                        // Multiplier
                        loop_block.i32_const(1); // We dont increase the capacity of nested vectors

                        if let Some(nested_fn) = nested_copy_function {
                            loop_block.call(nested_fn);
                        } else {
                            return Err(IntermediateTypeError::from(
                                RuntimeFunctionError::CouldNotLinkGeneric(
                                    RuntimeFunction::VecCopyLocal.name().to_owned(),
                                ),
                            ));
                        }
                    }
                    IntermediateType::IStruct {
                        module_id, index, ..
                    } => {
                        loop_block.load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );

                        let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;

                        struct_.copy_local_instructions(module, loop_block, compilation_ctx)?;
                    }

                    IntermediateType::IGenericStructInstance {
                        module_id,
                        index,
                        types,
                        ..
                    } => {
                        loop_block.load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );

                        let struct_ = compilation_ctx.get_struct_by_index(module_id, *index)?;
                        let struct_ = struct_.instantiate(types);

                        struct_.copy_local_instructions(module, loop_block, compilation_ctx)?;
                    }
                    IntermediateType::IEnum { .. } => {
                        loop_block.load(
                            compilation_ctx.memory_id,
                            LoadKind::I32 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 0,
                            },
                        );
                        let enum_ = compilation_ctx.get_enum_by_intermediate_type(inner)?;
                        inner_result =
                            enum_.copy_local_instructions(module, loop_block, compilation_ctx);
                    }

                    t => return Err(IntermediateTypeError::VectorUnnsuportedType(t.clone())),
                }

                // === Store result from stack into memory ===
                loop_block.store(
                    compilation_ctx.memory_id,
                    match inner {
                        IntermediateType::IU64 => StoreKind::I64 { atomic: false },
                        _ => StoreKind::I32 { atomic: false },
                    },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                // === index++ ===
                loop_block.local_get(index);
                loop_block.i32_const(1);
                loop_block.binop(BinaryOp::I32Add);
                loop_block.local_tee(index);

                // === Continue if index < len ===
                loop_block.local_get(len);
                loop_block.binop(BinaryOp::I32LtU);
                loop_block.br_if(loop_block.id());
                Ok(())
            })();
        });
    });

    inner_result?;

    // === Return pointer to copied vector ===
    builder.local_get(dst_ptr);

    Ok(function.finish(vec![src_ptr, multiplier], &mut module.funcs))
}

/// Compares two vectors for equality
///
/// # WASM Function Arguments
/// * `v1_ptr` (i32) - pointer to the first vector
/// * `v2_ptr` (i32) - pointer to the second vector
///
/// # WASM Function Returns
/// * `result` (i32) - 1 if vectors are equal, 0 otherwise
pub fn equality_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::VecEquality.get_generic_function_name(compilation_ctx, &[inner])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    // Arguments
    let v1_ptr = module.locals.add(ValType::I32);
    let v2_ptr = module.locals.add(ValType::I32);

    // Local variables
    let len = module.locals.add(ValType::I32);
    let result = module.locals.add(ValType::I32);
    builder.i32_const(1).local_set(result);

    // Load and compare the length of both vectors
    builder
        .local_get(v1_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_get(v2_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(len);

    // If both lengths are equal, we skip the capacity and compare element by element, otherwise we return false
    let mut inner_result: Result<(), IntermediateTypeError> = Ok(());
    builder.binop(BinaryOp::I32Eq).if_else(
        None,
        |then| {
            inner_result = (|| {
                let then_id = then.id();

                let i = module.locals.add(ValType::I32);
                then.i32_const(0).local_set(i);
                match inner {
                    IntermediateType::IBool
                    | IntermediateType::IU8
                    | IntermediateType::IU16
                    | IntermediateType::IU32
                    | IntermediateType::IU64 => {
                        // Call the generic equality function
                        let equality_f =
                            RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx))?;
                        then.skip_vec_header(v1_ptr)
                            .skip_vec_header(v2_ptr)
                            .local_get(len)
                            .i32_const(inner.wasm_memory_data_size()?)
                            .binop(BinaryOp::I32Mul)
                            .call(equality_f)
                            .local_set(result);
                    }
                    IntermediateType::IU128
                    | IntermediateType::IU256
                    | IntermediateType::IAddress
                    | IntermediateType::IVector(_)
                    | IntermediateType::IStruct { .. }
                    | IntermediateType::IGenericStructInstance { .. }
                    | IntermediateType::IEnum { .. }
                    | IntermediateType::IGenericEnumInstance { .. } => {
                        let mut loop_result: Result<(), IntermediateTypeError> = Ok(());
                        then.loop_(None, |loop_| {
                            loop_result = (|| {
                                //  Get the i-th element of both vectors and compare them
                                let data_size = inner.wasm_memory_data_size()?;
                                loop_.vec_elem_ptr(v1_ptr, i, data_size).load(
                                    compilation_ctx.memory_id,
                                    inner.load_kind()?,
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                );
                                loop_.vec_elem_ptr(v2_ptr, i, data_size).load(
                                    compilation_ctx.memory_id,
                                    inner.load_kind()?,
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                );

                                inner.load_equality_instructions(module, loop_, compilation_ctx)?;

                                // If they are not equal we set result to false and break the loop
                                loop_.if_else(
                                    None,
                                    |_| {},
                                    |else_| {
                                        else_.i32_const(0).local_set(result).br(then_id);
                                    },
                                );

                                // === index++ ===
                                loop_.local_get(i);
                                loop_.i32_const(1);
                                loop_.binop(BinaryOp::I32Add);
                                loop_.local_tee(i);

                                // === Continue if index < len ===
                                loop_.local_get(len);
                                loop_.binop(BinaryOp::I32LtU);
                                loop_.br_if(loop_.id());
                                Ok(())
                            })();
                        });

                        loop_result?;
                    }
                    IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                        return Err(IntermediateTypeError::FoundVectorOfReferences);
                    }
                    IntermediateType::ISigner => {
                        return Err(IntermediateTypeError::FoundVectorOfSigner);
                    }
                    IntermediateType::ITypeParameter(_) => {
                        return Err(IntermediateTypeError::FoundTypeParameter);
                    }
                }

                Ok(())
            })();
        },
        |else_| {
            else_.i32_const(0).local_set(result);
        },
    );

    inner_result.map_err(RuntimeFunctionError::from)?;

    builder.local_get(result);

    Ok(function.finish(vec![v1_ptr, v2_ptr], &mut module.funcs))
}

/// Converts raw bytes (bytesN) into a vector<u8>.
///
/// # WASM Function Arguments
/// * `bytes_ptr` (i32) - pointer to the raw bytes in memory
/// * `n` (i32) - number of bytes to convert
///
/// # WASM Function Returns
/// * i32 pointer to the newly created vector
pub fn bytes_to_vec_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::BytesToVec.name().to_owned())
        .func_body();

    let bytes_ptr = module.locals.add(ValType::I32);
    let n = module.locals.add(ValType::I32);

    let vector_ptr = module.locals.add(ValType::I32);

    let allocate_vector_with_header_function =
        RuntimeFunction::AllocateVectorWithHeader.get(module, Some(compilation_ctx))?;
    builder
        .local_get(n)
        .local_get(n)
        .i32_const(1)
        .call(allocate_vector_with_header_function)
        .local_set(vector_ptr);

    builder
        .skip_vec_header(vector_ptr)
        .local_get(bytes_ptr)
        .local_get(n)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    builder.local_get(vector_ptr);

    Ok(function.finish(vec![bytes_ptr, n], &mut module.funcs))
}
