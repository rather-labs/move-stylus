use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

use crate::CompilationContext;

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
        src_local: LocalId,
    ) {
        // === Local declarations ===
        let dst_local = module.locals.add(ValType::I32);
        let temp_local = module.locals.add(ValType::I32);

        let index = module.locals.add(ValType::I32);
        let len = module.locals.add(ValType::I32);

        let data_size = inner.stack_data_size() as i32;

        // === Read vector length ===
        builder.local_get(src_local);
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
            // === Compute address of copy element ===
            loop_block.local_get(index);
            loop_block.i32_const(data_size);
            loop_block.binop(BinaryOp::I32Mul);
            loop_block.i32_const(4); // skip vector length
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_get(src_local);
            loop_block.binop(BinaryOp::I32Add);

            match inner {
                IntermediateType::IBool
                | IntermediateType::IU8
                | IntermediateType::IU16
                | IntermediateType::IU32 => {
                    // Dont load the element, the copy_local_instructions expects an address!
                }
                IntermediateType::IU64 => {
                    // loop_block.unop(UnaryOp::I64ExtendUI32);
                }
                _ => {
                    // For heap, load the element (which is a pointer)
                    loop_block.load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    );
                }
            }

            loop_block.local_set(temp_local);

            // === Compute destination address of element ===
            loop_block.local_get(index);
            loop_block.i32_const(data_size);
            loop_block.binop(BinaryOp::I32Mul);
            loop_block.i32_const(4);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_get(dst_local);
            loop_block.binop(BinaryOp::I32Add);

            // === Copy element recursively ===
            inner.copy_local_instructions(module, loop_block, compilation_ctx, temp_local);

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

            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("Cannot VecImmBorrow an existing reference type");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;
    use walrus::{FunctionBuilder, FunctionId, MemoryId, Module, ModuleConfig, ValType};
    use wasmtime::{Engine, Instance, Linker, Module as WasmModule, Store, TypedFunc, WasmResults};

    use crate::memory::setup_module_memory;

    use super::*;

    fn build_module() -> (Module, FunctionId, MemoryId) {
        let config = ModuleConfig::new();
        let mut module = Module::with_config(config);
        let (allocator_func, memory_id) = setup_module_memory(&mut module);

        (module, allocator_func, memory_id)
    }

    fn setup_wasmtime_module<R: WasmResults>(
        module: &mut Module,
        initial_memory_data: Vec<u8>,
        function_name: &str,
    ) -> (Linker<()>, Instance, Store<()>, TypedFunc<(), R>) {
        let engine = Engine::default();
        let module = WasmModule::from_binary(&engine, &module.emit_wasm()).unwrap();

        let linker = Linker::new(&engine);

        let mut store = Store::new(&engine, ());
        let instance = linker.instantiate(&mut store, &module).unwrap();

        let entrypoint = instance
            .get_typed_func::<(), R>(&mut store, function_name)
            .unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        memory.write(&mut store, 0, &initial_memory_data).unwrap();

        (linker, instance, store, entrypoint)
    }

    fn test_vector(data: &[u8], inner_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module();

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
            setup_wasmtime_module::<i32>(&mut raw_module, vec![], "test_function");

        let result = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, 0);

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    fn test_vector_copy(data: &[u8], inner_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module();

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
        let src_local = raw_module.locals.add(ValType::I32);

        // Load the constant vector and store in local
        IVector::load_constant_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &mut data_iter.into_iter(),
            &compilation_ctx,
        );
        builder.local_set(src_local);

        // Copy the vector and return the new pointer
        IVector::copy_local_instructions(
            &inner_type,
            &mut raw_module,
            &mut builder,
            &compilation_ctx,
            src_local,
        );

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_copy_vector", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32>(&mut raw_module, vec![], "test_copy_vector");

        let result_ptr = entrypoint.call(&mut store, ()).unwrap();
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
        let (mut raw_module, allocator, memory_id) = build_module();

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
            setup_wasmtime_module::<i32>(&mut raw_module, vec![], "test_pack_vector");

        let result_ptr = entrypoint.call(&mut store, ()).unwrap();
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
            20u32.to_le_bytes().as_slice(),
            36u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            68u32.to_le_bytes().as_slice(),
        ]
        .concat();

        test_vector(&data, IntermediateType::IU128, &expected_result_bytes);
        test_vector_copy(&data, IntermediateType::IU128, &expected_copied_vector);
    }

    #[test]
    fn test_vector_u256() {
        let data = [
            &[4u8],
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
            U256::from(3u128).to_le_bytes::<32>().as_slice(),
            U256::from(4u128).to_le_bytes::<32>().as_slice(),
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
            U256::from(1u128).to_le_bytes::<32>().as_slice(),
            U256::from(2u128).to_le_bytes::<32>().as_slice(),
            U256::from(3u128).to_le_bytes::<32>().as_slice(),
            U256::from(4u128).to_le_bytes::<32>().as_slice(),
        ]
        .concat();

        let expected_copy_bytes = [
            4u32.to_le_bytes().as_slice(),
            // Pointers to memory
            20u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            84u32.to_le_bytes().as_slice(),
            116u32.to_le_bytes().as_slice(),
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
            20u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            84u32.to_le_bytes().as_slice(),
            116u32.to_le_bytes().as_slice(),
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
                &[4u8],
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                &[4u8],
                U256::from(5u128).to_le_bytes::<32>().as_slice(),
                U256::from(6u128).to_le_bytes::<32>().as_slice(),
                U256::from(7u128).to_le_bytes::<32>().as_slice(),
                U256::from(8u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat()
            .as_slice(),
        ]
        .concat();

        let expected_load_bytes = [
            2u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),  // pointer to first vector
            160u32.to_le_bytes().as_slice(), // pointer to second vector
            [
                4u32.to_le_bytes().as_slice(),
                // Pointers to memory
                32u32.to_le_bytes().as_slice(),
                64u32.to_le_bytes().as_slice(),
                96u32.to_le_bytes().as_slice(),
                128u32.to_le_bytes().as_slice(),
                // Referenced values
                U256::from(1u128).to_le_bytes::<32>().as_slice(),
                U256::from(2u128).to_le_bytes::<32>().as_slice(),
                U256::from(3u128).to_le_bytes::<32>().as_slice(),
                U256::from(4u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat() // 148 bytes
            .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                // Pointers to memory
                180u32.to_le_bytes().as_slice(),
                212u32.to_le_bytes().as_slice(),
                244u32.to_le_bytes().as_slice(),
                276u32.to_le_bytes().as_slice(),
                // Referenced values
                U256::from(5u128).to_le_bytes::<32>().as_slice(),
                U256::from(6u128).to_le_bytes::<32>().as_slice(),
                U256::from(7u128).to_le_bytes::<32>().as_slice(),
                U256::from(8u128).to_le_bytes::<32>().as_slice(),
            ]
            .concat() // 148 bytes
            .as_slice(),
        ]
        .concat(); // 308 bytes total

        let expected_copy_bytes = [
            2u32.to_le_bytes().as_slice(),
            320u32.to_le_bytes().as_slice(), // pointer to first copied vector: 308 + 4 + 4 + 4
            340u32.to_le_bytes().as_slice(), // pointer to second copied vector: 308 + 4 + 4 + 4 + 20
            [
                4u32.to_le_bytes().as_slice(),
                // Pointers to memory
                32u32.to_le_bytes().as_slice(),
                64u32.to_le_bytes().as_slice(),
                96u32.to_le_bytes().as_slice(),
                128u32.to_le_bytes().as_slice(),
            ]
            .concat()
            .as_slice(),
            [
                4u32.to_le_bytes().as_slice(),
                // Pointers to memory
                180u32.to_le_bytes().as_slice(),
                212u32.to_le_bytes().as_slice(),
                244u32.to_le_bytes().as_slice(),
                276u32.to_le_bytes().as_slice(),
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
