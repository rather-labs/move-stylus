use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::CompilationContext;
use crate::runtime::RuntimeFunction;
use crate::wasm_builder_extensions::WasmBuilderExtension;

use super::IntermediateType;

#[derive(Clone)]
pub struct IVector;

impl IVector {
    // Allocates memory for a vector with a header of 8 bytes.
    // First 4 bytes are the length, next 4 bytes are the capacity.
    pub fn allocate_vector_with_header(
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
        pointer: LocalId,
        len: LocalId,
        capacity: LocalId,
        data_size: i32,
    ) {

        builder
            .local_get(len)
            .local_get(capacity)
            .binop(BinaryOp::I32GtU)
            .if_else(
                None,
                |then_| {
                    then_.unreachable();  // Trap if len > capacity
                },
                |_| {}
            );

        // Allocate memory: capacity * element size + 8 bytes for header
        builder
            .local_get(capacity)
            .i32_const(data_size)
            .binop(BinaryOp::I32Mul)
            .i32_const(8)
            .binop(BinaryOp::I32Add)
            .call(compilation_ctx.allocator)
            .local_set(pointer);

        // Write length at offset 0
        builder.local_get(pointer).local_get(len).store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        // Write capacity at offset 4
        builder.local_get(pointer).local_get(capacity).store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 4,
            },
        );
    }

    pub fn load_constant_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        bytes: &mut std::vec::IntoIter<u8>,
        compilation_ctx: &CompilationContext,
    ) {
        let ptr_local = module.locals.add(ValType::I32);
        let len_local = module.locals.add(ValType::I32);

        // First byte is the length of the vector
        let len = bytes.next().unwrap();
        builder.i32_const(len as i32).local_set(len_local);

        let data_size: usize = inner.stack_data_size() as usize;

        // len + capacity + data_size * len
        let needed_bytes = 4 + 4 + data_size * (len as usize);

        IVector::allocate_vector_with_header(
            builder,
            compilation_ctx,
            ptr_local,
            len_local,
            len_local,
            data_size as i32,
        );

        let mut store_offset: u32 = 8;

        builder.local_get(ptr_local);
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

            builder.local_get(ptr_local);
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
        let capacity = module.locals.add(ValType::I32);

        let data_size = inner.stack_data_size() as i32;

        // === Read vector length ===
        builder
            .local_tee(src_local)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(len);

        // === Read vector capacity ===
        builder
            .local_get(src_local)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 4,
                },
            )
            .local_set(capacity);

        // Allocate memory and write length and capacity at the beginning
        IVector::allocate_vector_with_header(
            builder,
            compilation_ctx,
            dst_local,
            len,
            capacity,
            data_size,
        );

        // === Loop  ===
        builder.i32_const(0);
        builder.local_set(index);

        builder.loop_(None, |loop_block| {
            loop_block.vec_ptr_at(dst_local, index, data_size); // where to store the element
            loop_block.vec_ptr_at(src_local, index, data_size); // where to read the element

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
        // Local declarations
        let ptr_local = module.locals.add(ValType::I32);
        let len_local = module.locals.add(ValType::I32);
        let capacity_local = module.locals.add(ValType::I32);
        let temp_local = module.locals.add(inner.into());
        let data_size = inner.stack_data_size() as i32;

        // Set lenght
        builder.i32_const(num_elements).local_set(len_local);

        // Set capacity
        builder
            .i32_const(num_elements)
            .i32_const(2)
            .binop(BinaryOp::I32Mul)
            .local_set(capacity_local);

        IVector::allocate_vector_with_header(
            builder,
            compilation_ctx,
            ptr_local,
            len_local,
            capacity_local,
            data_size,
        );

        for i in 0..num_elements {
            builder.local_get(ptr_local);
            builder.swap(ptr_local, temp_local);

            // Store at computed address
            builder.store(
                compilation_ctx.memory_id,
                match inner.into() {
                    ValType::I64 => StoreKind::I64 { atomic: false },
                    ValType::I32 => StoreKind::I32 { atomic: false },
                    _ => panic!("Unsupported ValType"),
                },
                MemArg {
                    align: 0,
                    offset: (8 + (num_elements - 1 - i) * data_size) as u32,
                },
            );
        }

        builder.local_get(ptr_local);
    }

    pub fn add_vec_imm_borrow_instructions(
        inner: &IntermediateType,
        module: &mut Module,
        builder: &mut InstrSeqBuilder,
        compilation_ctx: &CompilationContext,
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
        builder
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            )
            .local_set(vector_address);

        // Trap if index >= length
        builder.block(None, |block| {
            block
                .local_get(vector_address)
                .load(
                    compilation_ctx.memory_id,
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

        // Reference to element
        let ref_local = module.locals.add(ValType::I32);
        builder
            .i32_const(4)
            .call(compilation_ctx.allocator)
            .local_tee(ref_local);

        // Compute element
        builder.vec_ptr_at(vector_address, index_i32, size);

        match inner {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                // Store element at ref address
            }

            IntermediateType::IVector(_)
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::ISigner
            | IntermediateType::IAddress => {
                // load pointer to value
                builder.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }

            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("VecImmBorrow operation is not allowed on reference types");
            }
        }

        builder.store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

        builder.local_get(ref_local);
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

    fn test_vector_pop_back(
        data: &[u8],
        inner_type: IntermediateType,
        expected_result_bytes: &[u8],
        expected_pop_stack: i32,
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

        // Mock mut ref layout. We store the address of the vector (4) at address 0
        let ptr = raw_module.locals.add(ValType::I32);
        builder.i32_const(4).call(allocator).local_tee(ptr);

        let data = data.to_vec();
        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut data.into_iter(),
            &compilation_ctx,
        );

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

        match inner_type {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IVector(_) => {
                let swap_f =
                    RuntimeFunction::VecPopBack32.get(&mut raw_module, Some(&compilation_ctx));
                builder.call(swap_f);
            }
            IntermediateType::IU64 => {
                let swap_f =
                    RuntimeFunction::VecPopBack64.get(&mut raw_module, Some(&compilation_ctx));
                builder.call(swap_f);
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("VecPopBack operation is not allowed on reference types");
            }
        }

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
        memory.read(&mut store, 4, &mut result_memory_data).unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    fn test_vector_swap(
        data: &[u8],
        inner_type: IntermediateType,
        expected_result_bytes: &[u8],
        idx1: i64,
        idx2: i64,
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

        // Mock mut ref
        let ptr = raw_module.locals.add(ValType::I32);
        builder.i32_const(4).call(allocator).local_tee(ptr);

        let data = data.to_vec();
        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut data.into_iter(),
            &compilation_ctx,
        );

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

        match inner_type {
            IntermediateType::IU64 => {
                let swap_f =
                    RuntimeFunction::VecSwap64.get(&mut raw_module, Some(&compilation_ctx));
                builder.call(swap_f);
            }
            _ => {
                let swap_f =
                    RuntimeFunction::VecSwap32.get(&mut raw_module, Some(&compilation_ctx));
                builder.call(swap_f);
            }
        }

        builder.i32_const(0);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![], "test_function", None);

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, 0);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory.read(&mut store, 4, &mut result_memory_data).unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    #[test]
    fn test_vector_bool() {
        let data = vec![4, 1, 0, 1, 0];
        let expected_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IBool, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IBool, &expected_bytes);
        test_vector_pop_back(&data, IntermediateType::IBool, &expected_pop_bytes, 0);
        test_vector_swap(&data, IntermediateType::IBool, &expected_swap_bytes, 0, 1);
    }

    #[test]
    fn test_vector_u8() {
        let data = vec![3, 1, 2, 3];

        let expected_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_swap_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IU8, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU8, &expected_bytes);
        test_vector_pop_back(&data, IntermediateType::IU8, &expected_pop_bytes, 3);
        test_vector_swap(&data, IntermediateType::IU8, &expected_swap_bytes, 0, 2);
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
        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU16, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU16, &expected_bytes);
        test_vector_pop_back(&data, IntermediateType::IU16, &expected_pop_bytes, 4);
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
            24u32.to_le_bytes().as_slice(),
            40u32.to_le_bytes().as_slice(),
            56u32.to_le_bytes().as_slice(),
            72u32.to_le_bytes().as_slice(),
            // Referenced values
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();

        let expected_copied_vector = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            112u32.to_le_bytes().as_slice(),
            128u32.to_le_bytes().as_slice(),
            144u32.to_le_bytes().as_slice(),
            160u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();
        let expected_pop_bytes = [
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            28u32.to_le_bytes().as_slice(),
            44u32.to_le_bytes().as_slice(),
            60u32.to_le_bytes().as_slice(),
            76u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();
        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            28u32.to_le_bytes().as_slice(),
            44u32.to_le_bytes().as_slice(),
            76u32.to_le_bytes().as_slice(),
            60u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vector(&data, IntermediateType::IU128, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU128, &expected_copied_vector);
        test_vector_pop_back(&data, IntermediateType::IU128, &expected_pop_bytes, 76);
        test_vector_swap(&data, IntermediateType::IU128, &expected_swap_bytes, 2, 3);
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
            16u32.to_le_bytes().as_slice(),
            48u32.to_le_bytes().as_slice(),
            // Referenced values
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_copy_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            // Pointers to memory
            96u32.to_le_bytes().as_slice(),
            128u32.to_le_bytes().as_slice(),
            // Referenced values
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_pop_bytes = [
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            20u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();
        let expected_swap_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            20u32.to_le_bytes().as_slice(),
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IU256, &expected_bytes);
        test_vector_copy(&data, IntermediateType::IU256, &expected_copy_bytes);
        test_vector_pop_back(&data, IntermediateType::IU256, &expected_pop_bytes, 52);
        test_vector_swap(&data, IntermediateType::IU256, &expected_swap_bytes, 0, 1);
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
            24u32.to_le_bytes().as_slice(),
            56u32.to_le_bytes().as_slice(),
            88u32.to_le_bytes().as_slice(),
            120u32.to_le_bytes().as_slice(),
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
            176u32.to_le_bytes().as_slice(),
            208u32.to_le_bytes().as_slice(),
            240u32.to_le_bytes().as_slice(),
            272u32.to_le_bytes().as_slice(),
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
            28u32.to_le_bytes().as_slice(),
            60u32.to_le_bytes().as_slice(),
            92u32.to_le_bytes().as_slice(),
            124u32.to_le_bytes().as_slice(),
            // Referenced values
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_swap_bytes = [
            4u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            124u32.to_le_bytes().as_slice(),
            60u32.to_le_bytes().as_slice(),
            92u32.to_le_bytes().as_slice(),
            28u32.to_le_bytes().as_slice(),
            // Referenced values
            U256::from(0x1111).to_be_bytes::<32>().as_slice(),
            U256::from(0x2222).to_be_bytes::<32>().as_slice(),
            U256::from(0x3333).to_be_bytes::<32>().as_slice(),
            U256::from(0x4444).to_be_bytes::<32>().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IAddress, &expected_load_bytes);
        test_vector_copy(&data, IntermediateType::IAddress, &expected_copy_bytes);
        test_vector_pop_back(&data, IntermediateType::IAddress, &expected_pop_bytes, 124);
        test_vector_swap(
            &data,
            IntermediateType::IAddress,
            &expected_swap_bytes,
            0,
            3,
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
            16u32.to_le_bytes().as_slice(), // pointer to first vector
            40u32.to_le_bytes().as_slice(), // pointer to second vector
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
            80u32.to_le_bytes().as_slice(),
            104u32.to_le_bytes().as_slice(),
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
            20u32.to_le_bytes().as_slice(),
            44u32.to_le_bytes().as_slice(),
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

        let expected_swap_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            44u32.to_le_bytes().as_slice(),
            20u32.to_le_bytes().as_slice(),
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
            IntermediateType::IVector(Box::new(IntermediateType::IU32)),
            &expected_load_bytes,
        );
        test_vector_copy(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU32)),
            &expected_copy_bytes,
        );
        test_vector_pop_back(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU32)),
            &expected_pop_bytes,
            44,
        );
        test_vector_swap(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU32)),
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
            16u32.to_le_bytes().as_slice(), // pointer to first vector
            96u32.to_le_bytes().as_slice(), // pointer to second vector
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                32u32.to_le_bytes().as_slice(),
                64u32.to_le_bytes().as_slice(),
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
                112u32.to_le_bytes().as_slice(),
                144u32.to_le_bytes().as_slice(),
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
            192u32.to_le_bytes().as_slice(),
            272u32.to_le_bytes().as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                // Pointers to memory
                208u32.to_le_bytes().as_slice(),
                240u32.to_le_bytes().as_slice(),
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
                288u32.to_le_bytes().as_slice(),
                320u32.to_le_bytes().as_slice(),
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
            20u32.to_le_bytes().as_slice(),
            100u32.to_le_bytes().as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                36u32.to_le_bytes().as_slice(),
                68u32.to_le_bytes().as_slice(),
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                116u32.to_le_bytes().as_slice(),
                148u32.to_le_bytes().as_slice(),
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();
        let expected_swap_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            100u32.to_le_bytes().as_slice(),
            20u32.to_le_bytes().as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                36u32.to_le_bytes().as_slice(),
                68u32.to_le_bytes().as_slice(),
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                2u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                116u32.to_le_bytes().as_slice(),
                148u32.to_le_bytes().as_slice(),
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
        test_vector_pop_back(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU256)),
            &expected_pop_bytes,
            100,
        );
        test_vector_swap(
            &data,
            IntermediateType::IVector(Box::new(IntermediateType::IU256)),
            &expected_swap_bytes,
            0,
            1,
        );
    }

    #[test]
    fn test_vec_pack_u8() {
        let element_bytes = vec![vec![10], vec![20], vec![30]];

        let expected_result_bytes = vec![
            3, 0, 0, 0, 6, 0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IU8,
            &expected_result_bytes,
        );
    }

    #[test]
    fn test_vec_pack_u32() {
        let element_bytes = vec![vec![10, 0, 0, 0], vec![20, 0, 0, 0], vec![30, 0, 0, 0]];

        let expected_result_bytes = vec![
            3, 0, 0, 0, 6, 0, 0, 0, 10, 0, 0, 0, 20, 0, 0, 0, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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

        let expected_result_bytes = vec![
            2, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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

        let expected_result_bytes = vec![
            3, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 32, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
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

        let expected_result_bytes = vec![
            2, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

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

        let expected_result_bytes = vec![
            3, 0, 0, 0, 6, 0, 0, 0, 0, 0, 0, 0, 80, 0, 0, 0, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];

        test_vector_pack(
            &element_bytes,
            IntermediateType::IVector(Box::new(IntermediateType::IU256)),
            &expected_result_bytes,
        );
    }
}
