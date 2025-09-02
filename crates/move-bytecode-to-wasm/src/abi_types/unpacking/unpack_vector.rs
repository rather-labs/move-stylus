use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    runtime::RuntimeFunction,
    translation::intermediate_types::{IntermediateType, vector::IVector},
};

use crate::CompilationContext;

use super::Unpackable;

impl IVector {
    pub fn add_unpack_instructions(
        inner: &IntermediateType,
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        // Big-endian to Little-endian
        let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None);

        let data_reader_pointer = module.locals.add(ValType::I32);

        // The ABI encoded value of a dynamic type is a reference to the location of the
        // values in the call data.
        // We are just assuming that the max value can fit in 32 bits, otherwise we cannot reference WASM memory
        // If the value is greater than 32 bits, the WASM program will panic
        for i in 0..7 {
            block.block(None, |inner_block| {
                let inner_block_id = inner_block.id();

                inner_block.local_get(reader_pointer);
                inner_block.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        // Abi encoded value is Big endian
                        offset: i * 4,
                    },
                );
                inner_block.i32_const(0);
                inner_block.binop(BinaryOp::I32Eq);
                inner_block.br_if(inner_block_id);
                inner_block.unreachable();
            });
        }
        block.local_get(reader_pointer);
        block.load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                // Abi encoded value is Big endian
                offset: 28,
            },
        );
        block.call(swap_i32_bytes_function);
        block.local_get(calldata_reader_pointer);
        block.binop(BinaryOp::I32Add);
        block.local_set(data_reader_pointer); // This references the vector actual data

        // The reader will only be incremented until the next argument
        block.local_get(reader_pointer);
        block.i32_const(32); // The size of the argument we just read
        block.binop(BinaryOp::I32Add);
        block.local_set(reader_pointer);

        // First 256 bits of the vector are the length
        // We are handling the length as u32 so the first 28 bytes are not needed
        // We need to ensure that they are zero to avoid runtime errors
        for i in 0..7 {
            block.block(None, |inner_block| {
                let inner_block_id = inner_block.id();

                inner_block.local_get(data_reader_pointer);
                inner_block.load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        // Abi encoded value is Big endian
                        offset: i * 4,
                    },
                );
                inner_block.i32_const(0);
                inner_block.binop(BinaryOp::I32Eq);
                inner_block.br_if(inner_block_id);
                inner_block.unreachable();
            });
        }

        // Vector length: current number of elements in the vector
        let length = module.locals.add(ValType::I32);

        block.local_get(data_reader_pointer);
        block.load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                // Abi encoded value is Big endian
                offset: 28,
            },
        );
        block.call(swap_i32_bytes_function);
        block.local_set(length);

        // increment data reader pointer
        block.local_get(data_reader_pointer);
        block.i32_const(32); // The size of the length in the ABI
        block.binop(BinaryOp::I32Add);
        block.local_set(data_reader_pointer);

        let vector_pointer = module.locals.add(ValType::I32);
        let writer_pointer = module.locals.add(ValType::I32);

        IVector::allocate_vector_with_header(
            block,
            compilation_ctx,
            vector_pointer,
            length,
            length,
            inner.stack_data_size() as i32,
        );
        block.local_get(vector_pointer);
        block.local_set(writer_pointer);

        // increment pointer
        block.local_get(writer_pointer);
        block.i32_const(8); // The size of the length + capacity written above
        block.binop(BinaryOp::I32Add);
        block.local_set(writer_pointer);

        // Copy elements
        let i = module.locals.add(ValType::I32);
        block.i32_const(0);
        block.local_set(i);

        let calldata_reader_pointer = module.locals.add(ValType::I32);
        block.local_get(data_reader_pointer);
        block.local_set(calldata_reader_pointer);

        block.loop_(None, |loop_block| {
            let loop_block_id = loop_block.id();

            loop_block.local_get(writer_pointer);
            // This will leave in the stack [pointer/value i32/i64, length i32]
            inner.add_unpack_instructions(
                loop_block,
                module,
                data_reader_pointer,
                calldata_reader_pointer,
                compilation_ctx,
            );

            // store the value
            if inner.stack_data_size() == 4 {
                loop_block.store(
                    compilation_ctx.memory_id,
                    StoreKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            } else if inner.stack_data_size() == 8 {
                loop_block.store(
                    compilation_ctx.memory_id,
                    StoreKind::I64 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            } else {
                unreachable!("Unsupported type size");
            }

            // increment writer pointer
            loop_block.local_get(writer_pointer);
            loop_block.i32_const(inner.stack_data_size() as i32);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_set(writer_pointer);

            // increment i
            loop_block.local_get(i);
            loop_block.i32_const(1);
            loop_block.binop(BinaryOp::I32Add);
            loop_block.local_tee(i);

            loop_block.local_get(length);
            loop_block.binop(BinaryOp::I32LtU);
            loop_block.br_if(loop_block_id);
        });

        // returned values
        block.local_get(vector_pointer);
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{U256, address};
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    use super::*;

    fn test_vec_unpacking(data: &[u8], int_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(Some(data.len() as i32));
        let compilation_ctx = test_compilation_context!(memory_id, allocator);
        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);
        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        // Args data should already be stored in memory
        int_type.add_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            args_pointer,
            calldata_reader_pointer,
            &compilation_ctx,
        );

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let global_next_free_memory_pointer = instance
            .get_global(&mut store, "global_next_free_memory_pointer")
            .unwrap();

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();
        assert_eq!(result, data.len() as i32);
        let global_next_free_memory_pointer = global_next_free_memory_pointer
            .get(&mut store)
            .i32()
            .unwrap();
        assert_eq!(
            global_next_free_memory_pointer,
            (expected_result_bytes.len() + data.len()) as i32
        );
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result_bytes.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u8_empty() {
        type SolType = sol!((uint8[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params::<(Vec<u8>,)>(&(vec![],));
        let expected_result_bytes =
            [0u64.to_le_bytes().as_slice(), 0u64.to_le_bytes().as_slice()].concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u8() {
        type SolType = sol!((uint8[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u16() {
        type SolType = sol!((uint16[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU16));

        let data = SolType::abi_encode_params(&(vec![1, 2],));
        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u32() {
        type SolType = sol!((uint32[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u64() {
        type SolType = sol!((uint64[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u64.to_le_bytes().as_slice(),
            2u64.to_le_bytes().as_slice(),
            3u64.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u128() {
        type SolType = sol!((uint128[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(vec![1, 2, 3],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((data.len() + 20) as u32).to_le_bytes().as_slice(),
            ((data.len() + 36) as u32).to_le_bytes().as_slice(),
            ((data.len() + 52) as u32).to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_u256() {
        type SolType = sol!((uint256[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IU256));

        let data =
            SolType::abi_encode_params(&(vec![U256::from(1), U256::from(2), U256::from(3)],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((data.len() + 20) as u32).to_le_bytes().as_slice(),
            ((data.len() + 52) as u32).to_le_bytes().as_slice(),
            ((data.len() + 84) as u32).to_le_bytes().as_slice(),
            U256::from(1).to_le_bytes::<32>().as_slice(),
            U256::from(2).to_le_bytes::<32>().as_slice(),
            U256::from(3).to_le_bytes::<32>().as_slice(),
        ]
        .concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_address() {
        type SolType = sol!((address[],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IAddress));

        let data = SolType::abi_encode_params(&(vec![
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
            address!("0x1234567890abcdef1234567890abcdef12345678"),
        ],));
        let expected_result_bytes = [
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            ((data.len() + 20) as u32).to_le_bytes().as_slice(),
            ((data.len() + 52) as u32).to_le_bytes().as_slice(),
            ((data.len() + 84) as u32).to_le_bytes().as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
            &[0; 12],
            address!("0x1234567890abcdef1234567890abcdef12345678").as_slice(),
        ]
        .concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_vector_u32() {
        type SolType = sol!((uint32[][],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU32,
        ))));

        let data = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));

        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            ((data.len() + 16) as u32).to_le_bytes().as_slice(),
            ((data.len() + 36) as u32).to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }

    #[test]
    fn test_unpack_vector_vector_u128() {
        type SolType = sol!((uint128[][],));
        let int_type = IntermediateType::IVector(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU128,
        ))));

        let data = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));
        let expected_result_bytes = [
            2u32.to_le_bytes().as_slice(),                        // len
            2u32.to_le_bytes().as_slice(),                        // capacity
            ((data.len() + 16) as u32).to_le_bytes().as_slice(),  // first element pointer
            ((data.len() + 84) as u32).to_le_bytes().as_slice(),  // second element pointer
            3u32.to_le_bytes().as_slice(),                        // first element length
            3u32.to_le_bytes().as_slice(),                        // first element capacity
            ((data.len() + 36) as u32).to_le_bytes().as_slice(), // first element - first value pointer
            ((data.len() + 52) as u32).to_le_bytes().as_slice(), // first element - second value pointer
            ((data.len() + 68) as u32).to_le_bytes().as_slice(), // first element - third value pointer
            1u128.to_le_bytes().as_slice(),                      // first element - first value
            2u128.to_le_bytes().as_slice(),                      // first element - second value
            3u128.to_le_bytes().as_slice(),                      // first element - third value
            3u32.to_le_bytes().as_slice(),                       // second element length
            3u32.to_le_bytes().as_slice(),                       // second element capacity
            ((data.len() + 104) as u32).to_le_bytes().as_slice(), // second element - first value pointer
            ((data.len() + 120) as u32).to_le_bytes().as_slice(), // second element - second value pointer
            ((data.len() + 136) as u32).to_le_bytes().as_slice(), // second element - third value pointer
            4u128.to_le_bytes().as_slice(),                       // second element - first value
            5u128.to_le_bytes().as_slice(),                       // second element - second value
            6u128.to_le_bytes().as_slice(),                       // second element - third value
        ]
        .concat();
        test_vec_unpacking(&data, int_type, &expected_result_bytes);
    }
}
