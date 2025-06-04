use walrus::{
    InstrSeqBuilder, MemoryId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::{CompilationContext, runtime::RuntimeFunction};

use super::IntermediateType;

#[derive(Clone)]
pub struct IVector;

impl IVector {
    pub fn load_constant_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        compilation_ctx: &CompilationContext,
    ) {
        // First byte is the length of the vector
        let vec_len = bytes.next().unwrap();

        let data_size: usize = inner.stack_data_size() as usize;

        // Vec len as i32 + data size * vec len
        let needed_bytes = 4 + data_size * (vec_len as usize);

        let pointer = module.locals.add(ValType::I32);

        builder.i32_const(needed_bytes as i32);
        builder.call(compilation_ctx.allocator);
        builder.local_tee(pointer);

        // Store length
        builder.i32_const(vec_len as i32);
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        let mut store_offset: u32 = 4;

        builder.local_get(pointer);
        while (store_offset as usize) < needed_bytes {
            // Load the inner type
            inner.load_constant_instructions(module, builder, bytes, compilation_ctx);

            if data_size == 4 {
                // Store i32
                builder.store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: store_offset,
                    },
                );

                store_offset += 4;
            } else if data_size == 8 {
                // Store i64
                builder.store(
                    compilation_ctx.memory_id,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: store_offset,
                    },
                );

                store_offset += 8;
            } else {
                panic!("Unsupported data size for vector: {}", data_size);
            }

            builder.local_get(pointer);
        }

        assert_eq!(
            needed_bytes, store_offset as usize,
            "Store offset is not aligned with the needed bytes"
        );
    }

    pub fn copy_local_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
    ) {
        // === Local declarations ===
        let src_local = module.locals.add(ValType::I32);
        let dst_local = module.locals.add(ValType::I32);
        let index = module.locals.add(ValType::I32);
        let len = module.locals.add(ValType::I32);

        let data_size = inner.stack_data_size() as i32;

        // === Read vector length ===
        builder.local_tee(src_local);
        builder.load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );
        builder.local_tee(len);

        // === Allocate memory for copy ===
        builder.i32_const(data_size);
        builder.binop(BinaryOp::I32Mul);
        builder.i32_const(4); // +4 for length prefix
        builder.binop(BinaryOp::I32Add);
        builder.call(compilation_ctx.allocator);
        builder.local_tee(dst_local);

        // === Write length at beginning of new memory ===
        builder.local_get(len);
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // === Loop  ===
        builder.i32_const(0);
        builder.local_set(index);

        builder.loop_(None, |loop_block| {
            // === Compute destination address of element ===
            loop_block.local_get(index);
            loop_block.i32_const(data_size);
            loop_block.binop(BinaryOp::I32Mul);
            loop_block.i32_const(4);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_get(dst_local);
            loop_block.binop(BinaryOp::I32Add);

            // === Compute address of copy element ===
            loop_block.local_get(index);
            loop_block.i32_const(data_size);
            loop_block.binop(BinaryOp::I32Mul);
            loop_block.i32_const(4);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_get(src_local);
            loop_block.binop(BinaryOp::I32Add);

            match inner {
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32 => {
                    loop_block.load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                }
                IntermediateType::IU64 => {
                    loop_block.load(
                        compilation_ctx.memory_id,
                        LoadKind::I64 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                }
                IntermediateType::IU128 => {
                    loop_block.load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                    let src_elem_ptr = module.locals.add(ValType::I32);
                    loop_block.local_set(src_elem_ptr);

                    loop_block.i32_const(16);
                    loop_block.call(compilation_ctx.allocator);
                    let dst_elem_ptr = module.locals.add(ValType::I32);
                    loop_block.local_set(dst_elem_ptr);

                    for i in 0..2 {
                        loop_block
                            .local_get(dst_elem_ptr)
                            .local_get(src_elem_ptr)
                            .load(
                                compilation_ctx.memory_id,
                                LoadKind::I64 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset: i * 8,
                                },
                            );
                    }

                    for i in 0..2 {
                        loop_block.store(
                            compilation_ctx.memory_id,
                            StoreKind::I64 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 8 - i * 8,
                            },
                        );
                    }

                    loop_block.local_get(dst_elem_ptr);
                }
                IntermediateType::IU256 | IntermediateType::IAddress => {
                    let src_elem_ptr = module.locals.add(ValType::I32);
                    loop_block.load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                    loop_block.local_set(src_elem_ptr);

                    loop_block.i32_const(32);
                    loop_block.call(compilation_ctx.allocator);
                    let dst_elem_ptr = module.locals.add(ValType::I32);
                    loop_block.local_set(dst_elem_ptr);

                    for i in 0..4 {
                        loop_block
                            .local_get(dst_elem_ptr)
                            .local_get(src_elem_ptr)
                            .load(
                                compilation_ctx.memory_id,
                                LoadKind::I64 { atomic: false },
                                MemArg {
                                    align: 0,
                                    offset: i * 8,
                                },
                            );
                    }

                    for i in 0..4 {
                        loop_block.store(
                            compilation_ctx.memory_id,
                            StoreKind::I64 { atomic: false },
                            MemArg {
                                align: 0,
                                offset: 24 - i * 8,
                            },
                        );
                    }
                    loop_block.local_get(dst_elem_ptr);
                }
                IntermediateType::IVector(inner_) => {
                    loop_block.load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                    IVector::copy_local_instructions(inner_, module, loop_block, compilation_ctx);
                }
                _ => {
                    panic!("Unsupported vector type");
                }
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
        });

        // === Return pointer to copied vector ===
        builder.local_get(dst_local);
    }

    pub fn vec_pack_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        num_elements: i32,
    ) {
        let data_size = inner.stack_data_size() as i32;
        let ptr_local = module.locals.add(ValType::I32);
        let temp_local = module.locals.add(inner.into());

        // Total size = 4 + data_size * num_elements
        builder.i32_const(4 + data_size * num_elements);
        builder.call(compilation_ctx.allocator);
        builder.local_tee(ptr_local);

        // Write the length at offset 0
        builder.i32_const(num_elements);
        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        for i in 0..num_elements {
            builder.local_set(temp_local);
            builder.local_get(ptr_local);
            builder.local_get(temp_local);

            // Store at computed address
            builder.store(
                compilation_ctx.memory_id,
                match data_size {
                    4 => StoreKind::I32 { atomic: false },
                    8 => StoreKind::I64 { atomic: false },
                    _ => panic!("Unsupported element size for vec_pack"),
                },
                MemArg {
                    align: 0,
                    offset: (4 + (num_elements - 1 - i) * data_size) as u32,
                },
            );
        }

        builder.local_get(ptr_local);
    }

    pub fn add_vec_imm_borrow_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        memory: MemoryId,
    ) {
        let size = inner.stack_data_size() as i32;
        let index_i64 = module.locals.add(ValType::I64); // referenced element index
        builder.local_set(index_i64); // index is on top of stack (as i64)

        // Trap if index > u32::MAX
        builder.block(None, |block| {
            block
                .local_get(index_i64)
                .i64_const(0xFFFF_FFFF)
                .binop(BinaryOp::I64LeU);
            block.br_if(block.id());
            block.unreachable();
        });

        //  Cast index to i32
        let index_i32 = module.locals.add(ValType::I32);
        builder
            .local_get(index_i64)
            .unop(UnaryOp::I32WrapI64)
            .local_set(index_i32);

        // Set vector base address
        let vector_address = module.locals.add(ValType::I32);
        builder.local_set(vector_address);

        // Trap if index >= length
        builder.block(None, |block| {
            block
                .local_get(vector_address)
                .load(
                    memory,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_get(index_i32)
                .binop(BinaryOp::I32GtU);
            block.br_if(block.id());
            block.unreachable();
        });

        // Compute element
        builder
            .local_get(vector_address)
            .i32_const(4)
            .binop(BinaryOp::I32Add)
            .local_get(index_i32)
            .i32_const(size)
            .binop(BinaryOp::I32Mul)
            .binop(BinaryOp::I32Add);

        match inner {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                // pointer to value
            }

            IntermediateType::IVector(_)
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::ISigner
            | IntermediateType::IAddress => {
                // load pointer to value
                builder.load(
                    memory,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }

            IntermediateType::IRef(_) => {
                panic!("Cannot VecImmBorrow an existing reference type");
            }
        }
    }

    pub fn equality(
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        compilation_ctx: &CompilationContext,
        inner: &IntermediateType,
    ) {
        let v1_ptr = module.locals.add(ValType::I32);
        let v2_ptr = module.locals.add(ValType::I32);
        let size = module.locals.add(ValType::I32);

        // Load the size of both vectors
        builder
            .local_set(v1_ptr)
            .local_tee(v2_ptr)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_get(v1_ptr)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_tee(size);

        // And chech if they equal, if they are, we compare element by element, otherwise, we
        // return false
        builder.binop(BinaryOp::I32Eq).if_else(
            ValType::I32,
            |then| {
                match inner {
                    IntermediateType::IBool
                    | IntermediateType::IU8
                    | IntermediateType::IU16
                    | IntermediateType::IU32
                    | IntermediateType::IU64 => {
                        let equality_f_id =
                            RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));

                        // Set the size as size * stack_data_size + 4.
                        // 4 bytes extra are occupied by the length of the vector
                        then.local_get(size)
                            .i32_const(inner.stack_data_size() as i32)
                            .binop(BinaryOp::I32Mul)
                            .i32_const(4)
                            .binop(BinaryOp::I32Add)
                            .local_set(size);

                        // Call the generic equality function
                        then.local_get(v1_ptr)
                            .local_get(v2_ptr)
                            .local_get(size)
                            .call(equality_f_id);
                    }
                    t @ (IntermediateType::IU128
                    | IntermediateType::IU256
                    | IntermediateType::IAddress) => {
                        let equality_f_id =
                            RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));

                        let res = module.locals.add(ValType::I32);
                        let offset = module.locals.add(ValType::I32);

                        // Set res to true
                        then.i32_const(1).local_set(res);
                        // Set the pointers past the length
                        then.local_get(v1_ptr)
                            .i32_const(4)
                            .binop(BinaryOp::I32Add)
                            .local_set(v1_ptr)
                            .local_get(v2_ptr)
                            .i32_const(4)
                            .binop(BinaryOp::I32Add)
                            .local_set(v2_ptr);

                        // Set the size as the length * 4 (pointer size)
                        then.local_get(size)
                            .i32_const(4)
                            .binop(BinaryOp::I32Mul)
                            .local_set(size);

                        // We must follow pointer by pointer and use the equality function
                        then.block(None, |block| {
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

                                // Load the heap size of the stack
                                loop_.i32_const(if *t == IntermediateType::IU128 {
                                    16
                                } else {
                                    32
                                });

                                loop_
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

                        then.local_get(res);
                    }
                    IntermediateType::IVector(intermediate_type) => todo!(),
                    IntermediateType::IRef(intermediate_type) => todo!(),
                    IntermediateType::ISigner => {
                        panic!("should not be possible to have a vector of signers")
                    }
                }
            },
            |else_| {
                else_.i32_const(0);
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::test_tools::{build_module, setup_wasmtime_module};
    use alloy_primitives::U256;
    use walrus::{FunctionBuilder, ValType};

    use super::*;

    fn test_vector(data: &[u8], inner_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(None);

        let compilation_ctx = CompilationContext {
            constants: &[],
            functions_arguments: &[],
            functions_returns: &[],
            module_signatures: &[],
            memory_id,
            allocator,
        };

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut builder = function_builder.func_body();

        let data = data.to_vec();
        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut data.into_iter(),
            &compilation_ctx,
        );

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, 0);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    fn test_vector_copy(data: &[u8], inner_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(None);

        let compilation_ctx = CompilationContext {
            constants: &[],
            functions_arguments: &[],
            functions_returns: &[],
            module_signatures: &[],
            memory_id,
            allocator,
        };

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let mut builder = function_builder.func_body();

        let data_iter = data.to_vec();

        // Load the constant vector and store in local
        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut data_iter.into_iter(),
            &compilation_ctx,
        );

        // Copy the vector and return the new pointer
        IVector::copy_local_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &compilation_ctx,
        );

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
        let (mut raw_module, allocator, memory_id) = build_module(None);

        let compilation_ctx = CompilationContext {
            constants: &[],
            functions_arguments: &[],
            functions_returns: &[],
            module_signatures: &[],
            memory_id,
            allocator,
        };

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let mut builder = function_builder.func_body();

        // Push elements to the stack
        for element_bytes in elements.iter() {
            let mut data_iter = element_bytes.clone().into_iter();
            inner_type.load_constant_instructions(
                &mut raw_module,
                &mut builder,
                &mut data_iter,
                &compilation_ctx,
            );
        }

        IVector::vec_pack_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &compilation_ctx,
            elements.len() as i32,
        );

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

    #[test]
    fn test_vector_bool() {
        let data = vec![4, 1, 0, 1, 0];
        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IBool, &expected_result_bytes);
        test_vector_copy(&data, IntermediateType::IBool, &expected_result_bytes);
    }

    #[test]
    fn test_vector_u8() {
        let data = vec![4, 1, 2, 3];

        let expected_load_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IU8, &expected_load_bytes);
        test_vector_copy(&data, IntermediateType::IU8, &expected_load_bytes);
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
        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU16, &expected_result_bytes);
        test_vector_copy(&data, IntermediateType::IU16, &expected_result_bytes);
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

        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU32, &expected_result_bytes);
        test_vector_copy(&data, IntermediateType::IU32, &expected_result_bytes);
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

        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
            4u64.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU64, &expected_result_bytes);
        test_vector_copy(&data, IntermediateType::IU64, &expected_result_bytes);
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

        let expected_result_bytes = [
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            20u32.to_le_bytes().as_slice(),
            36u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            68u32.to_le_bytes().as_slice(),
            // Referenced values
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_copied_vector = [
            4u32.to_le_bytes().as_slice(),
            104u32.to_le_bytes().as_slice(),
            120u32.to_le_bytes().as_slice(),
            136u32.to_le_bytes().as_slice(),
            152u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IU128, &expected_result_bytes);
        test_vector_copy(&data, IntermediateType::IU128, &expected_copied_vector);
    }

    #[test]
    fn test_vector_u256() {
        let data = [
            &[2u8],
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_load_bytes = [
            2u32.to_le_bytes().as_slice(),
            // Pointers to memory
            12u32.to_le_bytes().as_slice(),
            44u32.to_le_bytes().as_slice(),
            // Referenced values
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_copy_bytes = [
            2u32.to_le_bytes().as_slice(),
            // Pointers to memory
            88u32.to_le_bytes().as_slice(),
            120u32.to_le_bytes().as_slice(),
            // Referenced values
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IU256, &expected_load_bytes);
        test_vector_copy(&data, IntermediateType::IU256, &expected_copy_bytes);
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
            // Pointers to memory
            20u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            84u32.to_le_bytes().as_slice(),
            116u32.to_le_bytes().as_slice(),
            // Referenced values
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_copy_bytes = [
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            168u32.to_le_bytes().as_slice(),
            200u32.to_le_bytes().as_slice(),
            232u32.to_le_bytes().as_slice(),
            264u32.to_le_bytes().as_slice(),
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IAddress, &expected_load_bytes);
        test_vector_copy(&data, IntermediateType::IAddress, &expected_copy_bytes);
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
            12u32.to_le_bytes().as_slice(), // pointer to first vector
            32u32.to_le_bytes().as_slice(), // pointer to second vector
            [
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
            64u32.to_le_bytes().as_slice(), // pointer to first copied vector: 52 + 4 + 4 + 4
            84u32.to_le_bytes().as_slice(), // pointer to second copied vector: 52 + 4 + 4 + 4 + 20
            [
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
            IntermediateType::IVector(Box::new(IntermediateType::IU32)),
            &expected_load_bytes,
        );
        test_vector_copy(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU32)),
            &expected_copy_bytes,
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
            12u32.to_le_bytes().as_slice(), // pointer to first vector
            88u32.to_le_bytes().as_slice(), // pointer to second vector
            [
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                24u32.to_le_bytes().as_slice(),
                56u32.to_le_bytes().as_slice(),
                // Referenced values
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat() // 148 bytes
            .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                100u32.to_le_bytes().as_slice(),
                132u32.to_le_bytes().as_slice(),
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
            176u32.to_le_bytes().as_slice(),
            252u32.to_le_bytes().as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                188u32.to_le_bytes().as_slice(),
                220u32.to_le_bytes().as_slice(),
                // Referenced values
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                264u32.to_le_bytes().as_slice(),
                296u32.to_le_bytes().as_slice(),
                //Referenced values
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        test_vector(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU256)),
            &expected_load_bytes,
        );
        test_vector_copy(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU256)),
            &expected_copy_bytes,
        );
    }

    #[test]
    fn test_vec_pack_u8() {
        let element_bytes = vec![vec![10], vec![20], vec![30]];

        let expected_result_bytes = vec![3, 0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30, 0, 0, 0];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IU8,
            &expected_result_bytes,
        );
    }

    #[test]
    fn test_vec_pack_u32() {
        let element_bytes = vec![vec![10, 0, 0, 0], vec![20, 0, 0, 0], vec![30, 0, 0, 0]];

        let expected_result_bytes = vec![3, 0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30, 0, 0, 0];

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
            vec![3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        ];

        let expected_result_bytes = vec![
            4, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 32, 0, 0, 0, 48, 0, 0, 0,
        ];

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

        let expected_result_bytes = vec![3, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 64, 0, 0, 0];

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
            vec![2, 0, 0, 0, 30, 0, 0, 0, 40, 0, 0, 0],
        ];

        let expected_result_bytes = vec![3, 0, 0, 0, 0, 0, 0, 0, 12, 0, 0, 0, 24, 0, 0, 0];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IVector(Box::new(IntermediateType::IU32)),
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

        let expected_result_bytes = vec![3, 0, 0, 0, 0, 0, 0, 0, 76, 0, 0, 0, 152, 0, 0, 0];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IVector(Box::new(IntermediateType::IU256)),
            &expected_result_bytes,
        );
    }
}
