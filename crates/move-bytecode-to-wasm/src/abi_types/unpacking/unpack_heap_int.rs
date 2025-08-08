use alloy_sol_types::{SolType, sol_data};
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

use crate::{
    CompilationContext,
    runtime::RuntimeFunction,
    translation::intermediate_types::{
        address::IAddress,
        heap_integers::{IU128, IU256},
    },
};

impl IU128 {
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        _calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let encoded_size =
            sol_data::Uint::<128>::ENCODED_SIZE.expect("U128 should have a fixed size") as i32;

        // The data is padded 16 bytes to the right
        block
            .local_get(reader_pointer)
            .i32_const(16)
            .binop(BinaryOp::I32Add);

        let unpacked_pointer = module.locals.add(ValType::I32);
        block
            .i32_const(16)
            .call(compilation_ctx.allocator)
            .local_tee(unpacked_pointer);

        // Big-endian to Little-endian
        let swap_i128_bytes_function =
            RuntimeFunction::SwapI128Bytes.get(module, Some(compilation_ctx));
        block.call(swap_i128_bytes_function);

        // Increment reader pointer
        block
            .local_get(reader_pointer)
            .i32_const(encoded_size)
            .binop(BinaryOp::I32Add)
            .local_set(reader_pointer);

        block.local_get(unpacked_pointer);
    }
}

impl IU256 {
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        _calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let encoded_size =
            sol_data::Uint::<256>::ENCODED_SIZE.expect("U256 should have a fixed size") as i32;

        block.local_get(reader_pointer);

        let unpacked_pointer = module.locals.add(ValType::I32);
        block
            .i32_const(32)
            .call(compilation_ctx.allocator)
            .local_tee(unpacked_pointer);

        // Big-endian to Little-endian
        let swap_i256_bytes_function =
            RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));
        block.call(swap_i256_bytes_function);

        // Increment reader pointer
        block
            .local_get(reader_pointer)
            .i32_const(encoded_size)
            .binop(BinaryOp::I32Add)
            .local_set(reader_pointer);

        block.local_get(unpacked_pointer);
    }
}

impl IAddress {
    /// Address is packed as a u160, but endianness is not relevant
    pub fn add_unpack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        _calldata_reader_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        let encoded_size =
            sol_data::Address::ENCODED_SIZE.expect("Address should have a fixed size");

        block.i32_const(32);
        block.call(compilation_ctx.allocator);

        let unpacked_pointer = module.locals.add(ValType::I32);
        block.local_set(unpacked_pointer);

        for i in 0..4 {
            block.local_get(unpacked_pointer);
            block.local_get(reader_pointer);
            block.load(
                compilation_ctx.memory_id,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );

            block.store(
                compilation_ctx.memory_id,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
        }

        // increment reader pointer
        block.local_get(reader_pointer);
        block.i32_const(encoded_size as i32);
        block.binop(BinaryOp::I32Add);
        block.local_set(reader_pointer);

        block.local_get(unpacked_pointer);
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256, address};
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::unpacking::Unpackable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    fn test_uint(data: &[u8], int_type: IntermediateType, expected_result_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(Some(data.len() as i32));
        let compilation_ctx = test_compilation_context!(memory_id, allocator);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let args_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();
        func_body.i32_const(0);
        func_body.local_set(args_pointer);

        // Args data should already be stored in memory
        int_type.add_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            args_pointer,
            args_pointer,
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
    fn test_unpack_u128() {
        type IntType = u128;
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IU128;

        let data = SolType::abi_encode_params(&(88,));
        test_uint(&data, int_type.clone(), &88u128.to_le_bytes());

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(&data, int_type.clone(), &IntType::MAX.to_le_bytes()); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(&data, int_type.clone(), &IntType::MIN.to_le_bytes()); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(&data, int_type.clone(), &(IntType::MAX - 1).to_le_bytes()); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_u256() {
        type IntType = U256;
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IU256;

        let data = SolType::abi_encode_params(&(U256::from(88),));
        test_uint(&data, int_type.clone(), &U256::from(88).to_le_bytes::<32>());

        let data = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(&data, int_type.clone(), &IntType::MAX.to_le_bytes::<32>()); // max

        let data = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(&data, int_type.clone(), &IntType::MIN.to_le_bytes::<32>()); // min

        let data = SolType::abi_encode_params(&(IntType::MAX - U256::from(1),));
        test_uint(
            &data,
            int_type.clone(),
            &(IntType::MAX - U256::from(1)).to_le_bytes::<32>(),
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_unpack_address() {
        type SolType = sol!((address,));
        let int_type = IntermediateType::IAddress;

        let data = SolType::abi_encode_params(&(Address::ZERO,));
        test_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0x1234567890abcdef1234567890abcdef12345678"),));
        test_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        test_uint(&data, int_type.clone(), &data);

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE"),));
        test_uint(&data, int_type.clone(), &data);
    }
}
