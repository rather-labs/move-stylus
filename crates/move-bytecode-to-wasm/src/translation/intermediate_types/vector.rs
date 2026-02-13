use super::{IntermediateType, error::IntermediateTypeError};
use crate::{
    CompilationContext, data::RuntimeErrorData, error::RuntimeError, runtime::RuntimeFunction,
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    InstrSeqBuilder, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

#[derive(Clone)]
pub struct IVector;

impl IVector {
    pub fn load_constant_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::slice::Iter<'_, u8>,
        compilation_ctx: &CompilationContext,
    ) -> Result<(), IntermediateTypeError> {
        let ptr_local = module.locals.add(ValType::I32);
        let len_local = module.locals.add(ValType::I32);

        // First byte is the length of the vector
        let len = bytes
            .next()
            .ok_or(IntermediateTypeError::EmptyBytesInVector)?;
        builder.i32_const(*len as i32).local_set(len_local);

        let data_size: usize = inner.wasm_memory_data_size()? as usize;

        // len + capacity + data_size * len
        let needed_bytes = 4 + 4 + data_size * (*len as usize);

        let allocate_vector_with_header_function =
            RuntimeFunction::AllocateVectorWithHeader.get(module, Some(compilation_ctx), None)?;

        builder
            .local_get(len_local)
            .local_get(len_local)
            .i32_const(data_size as i32)
            .call(allocate_vector_with_header_function)
            .local_set(ptr_local);

        let mut store_offset: u32 = 8;

        builder.local_get(ptr_local);
        while (store_offset as usize) < needed_bytes {
            // Load the inner type
            inner.load_constant_instructions(module, builder, bytes, compilation_ctx)?;

            builder.store(
                compilation_ctx.memory_id,
                inner.store_kind()?,
                MemArg {
                    align: 0,
                    offset: store_offset,
                },
            );

            store_offset += data_size as u32;

            builder.local_get(ptr_local);
        }

        if needed_bytes != store_offset as usize {
            return Err(IntermediateTypeError::VectorStoreOffsetNotAligned {
                needed: needed_bytes,
                actual: store_offset as usize,
            });
        }

        Ok(())
    }

    pub fn vec_pack_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        num_elements: i32,
    ) -> Result<(), IntermediateTypeError> {
        // Local declarations
        let ptr_local = module.locals.add(ValType::I32);
        let len_local = module.locals.add(ValType::I32);
        let data_size = inner.wasm_memory_data_size()?;
        let allocate_vector_with_header_function =
            RuntimeFunction::AllocateVectorWithHeader.get(module, Some(compilation_ctx), None)?;

        if num_elements == 0 {
            // Set length
            builder.i32_const(0).local_set(len_local);

            builder
                .local_get(len_local)
                .local_get(len_local)
                .i32_const(data_size)
                .call(allocate_vector_with_header_function)
                .local_set(ptr_local);
        } else {
            // Set length
            builder.i32_const(num_elements).local_set(len_local);

            builder
                .local_get(len_local)
                .local_get(len_local)
                .i32_const(data_size)
                .call(allocate_vector_with_header_function)
                .local_set(ptr_local);

            let temp_local = module.locals.add(inner.try_into()?);
            for i in 0..num_elements {
                builder.local_get(ptr_local);
                builder.swap(ptr_local, temp_local);

                // Store at computed address
                builder.store(
                    compilation_ctx.memory_id,
                    inner.store_kind()?,
                    MemArg {
                        align: 0,
                        offset: (8 + (num_elements - 1 - i) * data_size) as u32,
                    },
                );
            }
        }

        builder.local_get(ptr_local);

        Ok(())
    }

    pub fn vec_unpack_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        runtime_error_data: &mut RuntimeErrorData,
        length: u64,
    ) -> Result<(), IntermediateTypeError> {
        let vec_ptr = module.locals.add(ValType::I32);
        builder.local_set(vec_ptr);

        // Verify the vector's in-memory length matches the VecUnpack expected length.
        // A mismatch indicates `destroy_empty` was called on a non-empty vector (only known case so far), so we abort with `VectorNotEmpty`.
        builder.block(None, |block| {
            let block_id = block.id();

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
                .i32_const(length as i32)
                .binop(BinaryOp::I32Eq)
                .br_if(block_id);

            block.return_error(
                module,
                compilation_ctx,
                Some(ValType::I32),
                runtime_error_data,
                RuntimeError::VectorNotEmpty,
            );
        });

        let i = module.locals.add(ValType::I32);
        builder.i32_const(0).local_set(i);

        let mut inner_result = Ok(());
        builder.block(None, |block| {
            let exit_loop_id = block.id();

            block.loop_(None, |loop_| {
                let loop_id = loop_.id();

                inner_result = (|| {
                    loop_
                        .local_get(i)
                        .i32_const(length as i32)
                        .binop(BinaryOp::I32GeU)
                        .br_if(exit_loop_id);

                    match inner {
                        IntermediateType::IBool
                        | IntermediateType::IU8
                        | IntermediateType::IU16
                        | IntermediateType::IU32
                        | IntermediateType::IU64
                        | IntermediateType::IU128
                        | IntermediateType::IU256
                        | IntermediateType::IAddress
                        | IntermediateType::IVector(_)
                        | IntermediateType::IStruct { .. }
                        | IntermediateType::IGenericStructInstance { .. }
                        | IntermediateType::IEnum { .. }
                        | IntermediateType::IGenericEnumInstance { .. } => {
                            loop_
                                .vec_elem_ptr(vec_ptr, i, inner.wasm_memory_data_size()?)
                                .load(
                                    compilation_ctx.memory_id,
                                    inner.load_kind()?,
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                );
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

                    loop_
                        .local_get(i)
                        .i32_const(1)
                        .binop(BinaryOp::I32Add)
                        .local_set(i);

                    loop_.br(loop_id);

                    Ok(())
                })();
            });
        });
        inner_result?;

        Ok(())
    }

    pub fn vec_borrow_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        runtime_error_data: &mut RuntimeErrorData,
        caller_return_type: Option<ValType>,
    ) -> Result<(), IntermediateTypeError> {
        let downcast_f = RuntimeFunction::DowncastU64ToU32.get(
            module,
            Some(compilation_ctx),
            Some(runtime_error_data),
        )?;

        match inner {
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                return Err(IntermediateTypeError::FoundVectorOfReferences);
            }

            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                builder.call_runtime_function(
                    compilation_ctx,
                    downcast_f,
                    &RuntimeFunction::DowncastU64ToU32,
                    caller_return_type,
                );
                builder.i32_const(0);
            }

            IntermediateType::IVector(_)
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::ISigner
            | IntermediateType::IAddress
            | IntermediateType::IStruct { .. }
            | IntermediateType::IGenericStructInstance { .. }
            | IntermediateType::IEnum { .. }
            | IntermediateType::IGenericEnumInstance { .. } => {
                builder.call_runtime_function(
                    compilation_ctx,
                    downcast_f,
                    &RuntimeFunction::DowncastU64ToU32,
                    caller_return_type,
                );
                builder.i32_const(1);
            }
            IntermediateType::ITypeParameter(_) => {
                return Err(IntermediateTypeError::FoundTypeParameter);
            }
        }

        builder.i32_const(inner.wasm_memory_data_size()?);

        let borrow_f = RuntimeFunction::VecBorrow.get(
            module,
            Some(compilation_ctx),
            Some(runtime_error_data),
        )?;
        builder.call_runtime_function(
            compilation_ctx,
            borrow_f,
            &RuntimeFunction::VecBorrow,
            caller_return_type,
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        data::RuntimeErrorData,
        test_compilation_context,
        test_tools::INITIAL_MEMORY_OFFSET,
        test_tools::{build_module, setup_wasmtime_module},
    };
    use alloy_primitives::U256;
    use walrus::ir::{LoadKind, StoreKind, UnaryOp};
    use walrus::{FunctionBuilder, ValType};

    use super::*;

    fn test_vector(data: &[u8], inner_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut builder = function_builder.func_body();

        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut data.iter(),
            &compilation_ctx,
        )
        .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    fn test_vector_copy(data: &[u8], inner_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);

        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);

        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let mut builder = function_builder.func_body();

        // Load the constant vector and store in local
        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut data.iter(),
            &compilation_ctx,
        )
        .unwrap();

        // Set the capacity equal to the length in this case
        builder.i32_const(1);

        // Copy the vector and return the new pointer
        let copy_local_function = RuntimeFunction::VecCopyLocal
            .get_generic(
                &mut raw_module,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                &[&inner_type],
            )
            .unwrap();

        builder.call(copy_local_function);
        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_copy_vector", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_copy_vector", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    fn test_vector_pack(
        elements: &[Vec<u8>],
        inner_type: IntermediateType,
        expected_result_bytes: &[u8],
    ) {
        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);
        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let mut builder = function_builder.func_body();

        // Push elements to the stack
        for element_bytes in elements.iter() {
            inner_type
                .load_constant_instructions(
                    &mut raw_module,
                    &mut builder,
                    &mut element_bytes.iter(),
                    &compilation_ctx,
                )
                .unwrap();
        }

        IVector::vec_pack_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &compilation_ctx,
            elements.len() as i32,
        )
        .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_pack_vector", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_pack_vector", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(result_memory_data, expected_result_bytes);
    }

    fn test_vector_pop_back(
        data: &[u8],
        inner_type: IntermediateType,
        expected_result_bytes: &[u8],
        expected_pop_stack: i32,
    ) {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);

        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut builder = function_builder.func_body();

        // Mock mut ref layout. We store the address of the vector (4) at address 0
        let ptr = raw_module.locals.add(ValType::I32);
        builder.i32_const(4).call(allocator).local_tee(ptr);

        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut data.iter(),
            &compilation_ctx,
        )
        .unwrap();

        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // pop back
        builder.local_get(ptr); // this would be the mutable reference to the vector

        let pop_back_f = RuntimeFunction::VecPopBack
            .get_generic(
                &mut raw_module,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                &[&inner_type],
            )
            .unwrap();
        builder.call_runtime_function(
            &compilation_ctx,
            pop_back_f,
            &RuntimeFunction::VecPopBack,
            Some(ValType::I32),
        );

        if inner_type == IntermediateType::IU64 {
            builder.unop(UnaryOp::I32WrapI64);
        }

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, expected_pop_stack);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(
                &mut store,
                INITIAL_MEMORY_OFFSET as usize + 4,
                &mut result_memory_data,
            )
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    fn test_vector_push_back(
        vector_data: &[u8],
        element_data: &[u8],
        inner_type: IntermediateType,
        expected_result_bytes: &[u8],
    ) {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);

        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut builder = function_builder.func_body();

        // Mock mut ref to vector layout.
        // The first 4 bytes will hold a pointer to the original vector unpacked data
        let vec_ref = raw_module.locals.add(ValType::I32);
        builder.i32_const(4).call(allocator).local_tee(vec_ref); // vec_ref == 0

        // Load the vector data into memory.
        // When loading a vector constant, the capacity is set to be equal to the length.
        // A pointer to the vector is pushed to the stack.
        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut vector_data.iter(),
            &compilation_ctx,
        )
        .unwrap();

        // Stack:
        // [Vector pointer]
        // [Address where to store the pointer (*vec_ref)]

        // Store the vector pointer in the first 4 bytes of memory: [4 0 0 0]
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        builder.local_get(vec_ref);

        let element_pointer = raw_module.locals.add((&inner_type).try_into().unwrap());
        inner_type
            .load_constant_instructions(
                &mut raw_module,
                &mut builder,
                &mut element_data.iter(),
                &compilation_ctx,
            )
            .unwrap();
        builder.local_tee(element_pointer);

        // Stack:
        // [Element pointer]
        // [Reference to vector]

        // First push back copies the entire vector, increasing its capacity
        let push_back_f = RuntimeFunction::VecPushBack
            .get_generic(
                &mut raw_module,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                &[&inner_type],
            )
            .unwrap();
        builder.call(push_back_f);

        // Second push back pushes the element to the new copied vector, which has capacity
        builder
            .local_get(vec_ref)
            .local_get(element_pointer)
            .call(push_back_f);

        builder.local_get(vec_ref).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let global_next_free_memory_pointer = instance
            .get_global(&mut store, "global_next_free_memory_pointer")
            .unwrap();

        let _vector_pointer: i32 = entrypoint.call(&mut store, ()).unwrap();

        let global_next_free_memory_pointer = global_next_free_memory_pointer
            .get(&mut store)
            .i32()
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        let offset = global_next_free_memory_pointer as usize - expected_result_bytes.len();
        memory
            .read(&mut store, offset, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    fn test_vector_swap(
        data: &[u8],
        inner_type: IntermediateType,
        expected_result_bytes: &[u8],
        idx1: i64,
        idx2: i64,
    ) {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);

        let compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut builder = function_builder.func_body();

        // Mock mut ref
        let ptr = raw_module.locals.add(ValType::I32);
        builder.i32_const(4).call(allocator).local_tee(ptr);

        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut data.iter(),
            &compilation_ctx,
        )
        .unwrap();

        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        builder.local_get(ptr); // Mut ref
        builder.i64_const(idx1); // idx1
        builder.i64_const(idx2); // idx2

        let swap_f = RuntimeFunction::VecSwap
            .get_generic(
                &mut raw_module,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                &[&inner_type],
            )
            .unwrap();
        builder.call_runtime_function(
            &compilation_ctx,
            swap_f,
            &RuntimeFunction::VecSwap,
            Some(ValType::I32),
        );

        builder.i32_const(0);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, 0);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(
                &mut store,
                INITIAL_MEMORY_OFFSET as usize + 4,
                &mut result_memory_data,
            )
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    #[test]
    fn test_vector_bool() {
        let data = vec![4, 1, 0, 1, 0];
        let expected_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_push_bytes = [
            6u32.to_le_bytes().as_slice(),
            8u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
        ]
        .concat();
        let element_bytes = [1u8];

        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IBool, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IBool, &expected_bytes);
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IBool,
            &expected_push_bytes,
        );
        test_vector_pop_back(&data, IntermediateType::IBool, &expected_pop_bytes, 0);
        test_vector_swap(&data, IntermediateType::IBool, &expected_swap_bytes, 0, 1);
    }

    #[test]
    fn test_vector_u8() {
        let data = vec![3, 1, 2, 3];

        let expected_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_push_bytes = [
            5u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
            4u8.to_le_bytes().as_slice(),
            4u8.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
        ]
        .concat();

        let element_bytes = [4u8];

        let expected_swap_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IU8, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU8, &expected_bytes);
        test_vector_pop_back(&data, IntermediateType::IU8, &expected_pop_bytes, 3);
        test_vector_swap(&data, IntermediateType::IU8, &expected_swap_bytes, 0, 2);
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IU8,
            &expected_push_bytes,
        );
    }

    #[test]
    fn test_vector_u16() {
        let data = [
            &[4u8],
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
            3u16.to_le_bytes().as_slice(),
            4u16.to_le_bytes().as_slice(),
        ]
        .concat();
        let expected_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
            3u16.to_le_bytes().as_slice(),
            4u16.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
            3u16.to_le_bytes().as_slice(),
            4u16.to_le_bytes().as_slice(),
        ]
        .concat();

        let element_bytes = [5u16.to_le_bytes().as_slice()].concat();

        let expected_push_bytes = [
            6u32.to_le_bytes().as_slice(),
            8u32.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
            3u16.to_le_bytes().as_slice(),
            4u16.to_le_bytes().as_slice(),
            5u16.to_le_bytes().as_slice(),
            5u16.to_le_bytes().as_slice(),
            0u16.to_le_bytes().as_slice(),
            0u16.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            3u16.to_le_bytes().as_slice(),
            2u16.to_le_bytes().as_slice(),
            1u16.to_le_bytes().as_slice(),
            4u16.to_le_bytes().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IU16, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU16, &expected_bytes);
        test_vector_pop_back(&data, IntermediateType::IU16, &expected_pop_bytes, 4);
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IU16,
            &expected_push_bytes,
        );
        test_vector_swap(&data, IntermediateType::IU16, &expected_swap_bytes, 0, 2);
    }

    #[test]
    fn test_vector_u32() {
        let data = [
            &[4u8],
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();
        let expected_pop_bytes = [
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();
        let expected_push_bytes = [
            6u32.to_le_bytes().as_slice(),
            8u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
        ]
        .concat();
        let element_bytes = [5u32.to_le_bytes().as_slice()].concat();
        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU32, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU32, &expected_bytes);
        test_vector_pop_back(&data, IntermediateType::IU32, &expected_pop_bytes, 4);
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IU32,
            &expected_push_bytes,
        );
        test_vector_swap(&data, IntermediateType::IU32, &expected_swap_bytes, 1, 3);
    }

    #[test]
    fn test_vector_u64() {
        let data = [
            &[4u8],
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
            4u64.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
            4u64.to_le_bytes().as_slice(),
        ]
        .concat();
        let expected_pop_bytes = [
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
            4u64.to_le_bytes().as_slice(),
        ]
        .concat();
        let expected_push_bytes = [
            6u32.to_le_bytes().as_slice(),
            8u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
            4u64.to_le_bytes().as_slice(),
            5u64.to_le_bytes().as_slice(),
            5u64.to_le_bytes().as_slice(),
            0u64.to_le_bytes().as_slice(),
            0u64.to_le_bytes().as_slice(),
        ]
        .concat();
        let element_bytes = [5u64.to_le_bytes().as_slice()].concat();
        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            4u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU64, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU64, &expected_bytes);
        test_vector_pop_back(&data, IntermediateType::IU64, &expected_pop_bytes, 4);
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IU64,
            &expected_push_bytes,
        );
        test_vector_swap(&data, IntermediateType::IU64, &expected_swap_bytes, 0, 3);
    }

    #[test]
    fn test_vector_u128() {
        let data = [
            &[4u8],
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            ((INITIAL_MEMORY_OFFSET + 24) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 40) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 56) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 72) as u32)
                .to_le_bytes()
                .as_slice(),
            // Referenced values
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_copy_vector = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 112) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 128) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 144) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 160) as u32)
                .to_le_bytes()
                .as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 28) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 44) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 60) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 76) as u32)
                .to_le_bytes()
                .as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_push_bytes = [
            99u128.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            8u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 148) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 164) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 180) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 196) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 92) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 92) as u32)
                .to_le_bytes()
                .as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();
        let element_bytes = [99u128.to_le_bytes().as_slice()].concat();

        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 28) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 44) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 76) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 60) as u32)
                .to_le_bytes()
                .as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU128, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU128, &expected_copy_vector);
        test_vector_pop_back(
            &data,
            IntermediateType::IU128,
            &expected_pop_bytes,
            INITIAL_MEMORY_OFFSET + 76,
        );
        test_vector_swap(&data, IntermediateType::IU128, &expected_swap_bytes, 2, 3);
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IU128,
            &expected_push_bytes,
        );
    }

    #[test]
    fn test_vector_u256() {
        let data = [
            &[2u8],
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            // Pointers to memory
            ((INITIAL_MEMORY_OFFSET + 16) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 48) as u32)
                .to_le_bytes()
                .as_slice(),
            // Referenced values
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_copy_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            // Pointers to memory
            ((INITIAL_MEMORY_OFFSET + 96) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 128) as u32)
                .to_le_bytes()
                .as_slice(),
            // Referenced values
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 20) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 52) as u32)
                .to_le_bytes()
                .as_slice(),
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_push_bytes = [
            U256::from(99u128).to_le_bytes::<32>().as_slice(),
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 140) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 172) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 84) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 84) as u32)
                .to_le_bytes()
                .as_slice(),
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();
        let element_bytes = [U256::from(99u128).to_le_bytes::<32>().as_slice()].concat();

        let expected_swap_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 52) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 20) as u32)
                .to_le_bytes()
                .as_slice(),
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IU256, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU256, &expected_copy_bytes);
        test_vector_pop_back(
            &data,
            IntermediateType::IU256,
            &expected_pop_bytes,
            INITIAL_MEMORY_OFFSET + 52,
        );
        test_vector_swap(&data, IntermediateType::IU256, &expected_swap_bytes, 0, 1);
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IU256,
            &expected_push_bytes,
        );
    }

    #[test]
    fn test_vector_address() {
        let data = [
            &[4u8],
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_load_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            ((INITIAL_MEMORY_OFFSET + 24) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 56) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 88) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 120) as u32)
                .to_le_bytes()
                .as_slice(),
            // Referenced values
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_copy_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            ((INITIAL_MEMORY_OFFSET + 176) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 208) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 240) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 272) as u32)
                .to_le_bytes()
                .as_slice(),
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();
        let expected_pop_bytes = [
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            ((INITIAL_MEMORY_OFFSET + 28) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 60) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 92) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 124) as u32)
                .to_le_bytes()
                .as_slice(),
            // Referenced values
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_push_bytes = [
            U256::from(0x5555).to_be_bytes::<32>().as_slice(),
            6u32.to_le_bytes().as_slice(),
            8u32.to_le_bytes().as_slice(),
            // Pointers to memory
            ((INITIAL_MEMORY_OFFSET + 228) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 260) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 292) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 324) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 156) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 156) as u32)
                .to_le_bytes()
                .as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        let element_bytes = [U256::from(0x5555).to_be_bytes::<32>().as_slice()].concat();

        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            ((INITIAL_MEMORY_OFFSET + 124) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 60) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 92) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 28) as u32)
                .to_le_bytes()
                .as_slice(),
            // Referenced values
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IAddress, &expected_load_bytes);
        test_vector_copy(&data, IntermediateType::IAddress, &expected_copy_bytes);
        test_vector_pop_back(
            &data,
            IntermediateType::IAddress,
            &expected_pop_bytes,
            INITIAL_MEMORY_OFFSET + 124,
        );
        test_vector_swap(
            &data,
            IntermediateType::IAddress,
            &expected_swap_bytes,
            0,
            3,
        );
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IAddress,
            &expected_push_bytes,
        );
    }

    #[test]
    fn test_vector_vector_u32() {
        let data = [
            &[2u8],
            [
                &[4u8],
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                &[4u8],
                5u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                7u32.to_le_bytes().as_slice(),
                8u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        let expected_load_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 16) as u32)
                .to_le_bytes()
                .as_slice(), // pointer to first vector
            ((INITIAL_MEMORY_OFFSET + 40) as u32)
                .to_le_bytes()
                .as_slice(), // pointer to second vector
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                5u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                7u32.to_le_bytes().as_slice(),
                8u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat(); // 52 bytes total

        let expected_copy_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 80) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 104) as u32)
                .to_le_bytes()
                .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                5u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                7u32.to_le_bytes().as_slice(),
                8u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 20) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 44) as u32)
                .to_le_bytes()
                .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                5u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                7u32.to_le_bytes().as_slice(),
                8u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat(); // 52 bytes total

        let expected_push_bytes = [
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                101u32.to_le_bytes().as_slice(),
                102u32.to_le_bytes().as_slice(),
                103u32.to_le_bytes().as_slice(),
                104u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(), // push back element is loaded before the new vector!
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 116) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 140) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 68) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 68) as u32)
                .to_le_bytes()
                .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                5u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                7u32.to_le_bytes().as_slice(),
                8u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        let element_bytes = [
            &[4u8],
            101u32.to_le_bytes().as_slice(),
            102u32.to_le_bytes().as_slice(),
            103u32.to_le_bytes().as_slice(),
            104u32.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_swap_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 44) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 20) as u32)
                .to_le_bytes()
                .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                4u32.to_le_bytes().as_slice(),
                5u32.to_le_bytes().as_slice(),
                6u32.to_le_bytes().as_slice(),
                7u32.to_le_bytes().as_slice(),
                8u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        test_vector(
            &data,
            IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
            &expected_load_bytes,
        );
        test_vector_copy(
            &data,
            IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
            &expected_copy_bytes,
        );
        test_vector_pop_back(
            &data,
            IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
            &expected_pop_bytes,
            INITIAL_MEMORY_OFFSET + 44,
        );
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
            &expected_push_bytes,
        );
        test_vector_swap(
            &data,
            IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
            &expected_swap_bytes,
            0,
            1,
        );
    }

    #[test]
    fn test_vector_vector_u256() {
        let data = [
            &[2u8],
            [
                &[2u8],
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                &[2u8],
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        let expected_load_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 16) as u32)
                .to_le_bytes()
                .as_slice(), // pointer to first vector
            ((INITIAL_MEMORY_OFFSET + 96) as u32)
                .to_le_bytes()
                .as_slice(), // pointer to second vector
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                ((INITIAL_MEMORY_OFFSET + 32) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 64) as u32)
                    .to_le_bytes()
                    .as_slice(),
                // Referenced values
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat() // 148 bytes
            .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                ((INITIAL_MEMORY_OFFSET + 112) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 144) as u32)
                    .to_le_bytes()
                    .as_slice(),
                // Referenced values
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat() // 148 bytes
            .as_slice(),
        ]
        .concat(); // 308 bytes total

        let expected_copy_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 192) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 272) as u32)
                .to_le_bytes()
                .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                ((INITIAL_MEMORY_OFFSET + 208) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 240) as u32)
                    .to_le_bytes()
                    .as_slice(),
                // Referenced values
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                ((INITIAL_MEMORY_OFFSET + 288) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 320) as u32)
                    .to_le_bytes()
                    .as_slice(),
                //Referenced values
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 20) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 100) as u32)
                .to_le_bytes()
                .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                ((INITIAL_MEMORY_OFFSET + 36) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 68) as u32)
                    .to_le_bytes()
                    .as_slice(),
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                ((INITIAL_MEMORY_OFFSET + 116) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 148) as u32)
                    .to_le_bytes()
                    .as_slice(),
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        let expected_push_bytes = [
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                ((INITIAL_MEMORY_OFFSET + 196) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 228) as u32)
                    .to_le_bytes()
                    .as_slice(),
                //Referenced values
                U256::from(5u128).to_le_bytes::<32>().as_slice(),
                U256::from(6u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 284) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 364) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 180) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 180) as u32)
                .to_le_bytes()
                .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                ((INITIAL_MEMORY_OFFSET + 300) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 332) as u32)
                    .to_le_bytes()
                    .as_slice(),
                // Referenced values
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                ((INITIAL_MEMORY_OFFSET + 380) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 412) as u32)
                    .to_le_bytes()
                    .as_slice(),
                //Referenced values
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();
        let element_bytes = [
            &[2u8],
            U256::from(5u128).to_le_bytes::<32>().as_slice(),
            U256::from(6u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_swap_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((INITIAL_MEMORY_OFFSET + 100) as u32)
                .to_le_bytes()
                .as_slice(),
            ((INITIAL_MEMORY_OFFSET + 20) as u32)
                .to_le_bytes()
                .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                ((INITIAL_MEMORY_OFFSET + 36) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 68) as u32)
                    .to_le_bytes()
                    .as_slice(),
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                ((INITIAL_MEMORY_OFFSET + 116) as u32)
                    .to_le_bytes()
                    .as_slice(),
                ((INITIAL_MEMORY_OFFSET + 148) as u32)
                    .to_le_bytes()
                    .as_slice(),
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        test_vector(
            &data,
            IntermediateType::IVector(Arc::new(IntermediateType::IU256)),
            &expected_load_bytes,
        );
        test_vector_copy(
            &data,
            IntermediateType::IVector(Arc::new(IntermediateType::IU256)),
            &expected_copy_bytes,
        );
        test_vector_pop_back(
            &data,
            IntermediateType::IVector(Arc::new(IntermediateType::IU256)),
            &expected_pop_bytes,
            INITIAL_MEMORY_OFFSET + 100,
        );
        test_vector_push_back(
            &data,
            &element_bytes,
            IntermediateType::IVector(Arc::new(IntermediateType::IU256)),
            &expected_push_bytes,
        );
        test_vector_swap(
            &data,
            IntermediateType::IVector(Arc::new(IntermediateType::IU256)),
            &expected_swap_bytes,
            0,
            1,
        );
    }

    #[test]
    fn test_vec_pack_u8() {
        let element_bytes = vec![vec![10], vec![20], vec![30]];

        let expected_result_bytes = vec![3, 0, 0, 0, 3, 0, 0, 0, 10, 20, 30];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IU8,
            &expected_result_bytes,
        );
    }

    #[test]
    fn test_vec_pack_u16() {
        let element_bytes = vec![vec![10], vec![20], vec![30]];

        let expected_result_bytes = vec![3, 0, 0, 0, 3, 0, 0, 0, 10, 0, 20, 0, 30, 0];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IU16,
            &expected_result_bytes,
        );
    }

    #[test]
    fn test_vec_pack_u32() {
        let element_bytes = vec![vec![10, 0, 0, 0], vec![20, 0, 0, 0], vec![30, 0, 0, 0]];

        let expected_result_bytes = vec![
            3, 0, 0, 0, 3, 0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30, 0, 0, 0,
        ];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IU32,
            &expected_result_bytes,
        );
    }

    #[test]
    fn test_vec_pack_u128() {
        let element_bytes = vec![
            vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];

        let expected_result_bytes = vec![2, 0, 0, 0, 2, 0, 0, 0, 208, 7, 0, 0, 224, 7, 0, 0];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IU128,
            &expected_result_bytes,
        );
    }

    #[test]
    fn test_vec_pack_u256() {
        let element_bytes = vec![
            vec![
                1, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
            vec![
                2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
            vec![
                3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0,
            ],
        ];

        let expected_result_bytes = vec![
            3, 0, 0, 0, 3, 0, 0, 0, 208, 7, 0, 0, 240, 7, 0, 0, 16, 8, 0, 0,
        ];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IU256,
            &expected_result_bytes,
        );
    }

    #[test]
    fn test_vec_pack_vec_u32() {
        let element_bytes = vec![
            vec![2, 0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0],
            vec![2, 0, 0, 0, 30, 0, 0, 0, 40, 0, 0, 0],
        ];

        let expected_result_bytes = vec![2, 0, 0, 0, 2, 0, 0, 0, 208, 7, 0, 0, 224, 7, 0, 0];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
            &expected_result_bytes,
        );
    }

    #[test]
    fn test_vec_pack_vec_u256() {
        // Each inner vector has 2 elements of u256 (32 bytes each + 4 bytes for pointer) + 4 bytes for length = 76 bytes
        let element_bytes = vec![
            // First inner vector [1, 2]
            {
                let mut v = vec![2, 0, 0, 0];
                v.extend_from_slice(&[1; 32]);
                v.extend_from_slice(&[2; 32]);
                v
            },
            // Second inner vector [3, 4]
            {
                let mut v = vec![2, 0, 0, 0];
                v.extend_from_slice(&[3; 32]);
                v.extend_from_slice(&[4; 32]);
                v
            },
            // Third inner vector [5, 6]
            {
                let mut v = vec![2, 0, 0, 0];
                v.extend_from_slice(&[5; 32]);
                v.extend_from_slice(&[6; 32]);
                v
            },
        ];

        let expected_result_bytes = vec![
            3, 0, 0, 0, 3, 0, 0, 0, 208, 7, 0, 0, 32, 8, 0, 0, 112, 8, 0, 0,
        ];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IVector(Arc::new(IntermediateType::IU256)),
            &expected_result_bytes,
        );
    }
}
