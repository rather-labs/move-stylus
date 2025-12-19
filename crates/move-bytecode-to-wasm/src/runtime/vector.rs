use walrus::{
    FunctionBuilder, FunctionId, InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, ExtendedLoad, LoadKind, MemArg, StoreKind, UnaryOp},
};

use super::{RuntimeFunction, error::RuntimeFunctionError};
use crate::CompilationContext;
use crate::translation::intermediate_types::IntermediateType;
use crate::translation::intermediate_types::vector::IVector;
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
/// # Parameters
/// * `inner_type` - The intermediate type of the vector's inner elements
/// * `function_name` - The name to use for the generated WASM function
pub fn vec_swap_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner_type: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let element_size = inner_type.wasm_memory_data_size()? as i32;
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
/// # Parameters
/// * `inner_type` - The intermediate type of the vector's inner elements
pub fn vec_pop_back_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner_type: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let element_size = inner_type.wasm_memory_data_size()? as i32;
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

/// Converts raw bytes (bytesN) into a vector<u8>.
///
/// # WASM Function Arguments
/// * `bytes_ptr` (i32) - pointer to the raw bytes in memory
/// * `bytes_n` (i32) - number of bytes to convert
///
/// # WASM Function Returns
/// * i32 pointer to the newly created vector
pub fn bytes_to_vec_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::BytesToVec.name().to_owned())
        .func_body();

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

    function.finish(vec![bytes_ptr, bytes_n], &mut module.funcs)
}
