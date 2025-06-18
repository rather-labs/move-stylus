use walrus::{
    InstrSeqBuilder, LocalId, MemoryId, Module,
    ir::{LoadKind, MemArg, StoreKind},
};

use crate::{
    runtime::RuntimeFunction,
    translation::intermediate_types::{
        address::IAddress,
        heap_integers::{IU128, IU256},
        signer::ISigner,
    },
};

impl IU128 {
    pub fn add_pack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        memory: MemoryId,
    ) {
        // Little-endian to Big-endian
        let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None);

        for i in 0..2 {
            block.local_get(writer_pointer);
            block.local_get(local);

            // Load from right to left
            block.load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 8 - i * 8,
                },
            );
            block.call(swap_i64_bytes_function);

            // Store from left to right
            block.store(
                memory,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    // Abi is left-padded to 32 bytes
                    offset: 16 + i * 8,
                },
            );
        }
    }
}

impl IU256 {
    pub fn add_pack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        memory: MemoryId,
    ) {
        // Little-endian to Big-endian
        let swap_i64_bytes_function = RuntimeFunction::SwapI64Bytes.get(module, None);

        for i in 0..4 {
            block.local_get(writer_pointer);
            block.local_get(local);

            // Load from right to left
            block.load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 24 - i * 8,
                },
            );
            block.call(swap_i64_bytes_function);

            // Store from left to right
            block.store(
                memory,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
        }
    }
}

impl IAddress {
    /// Address is packed as a u160, but endianness is not relevant
    pub fn add_pack_instructions(
        block: &mut InstrSeqBuilder,
        _module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        memory: MemoryId,
    ) {
        for i in 0..4 {
            block.local_get(writer_pointer);
            block.local_get(local);

            block.load(
                memory,
                LoadKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
            block.store(
                memory,
                StoreKind::I64 { atomic: false },
                MemArg {
                    align: 0,
                    offset: i * 8,
                },
            );
        }
    }
}

impl ISigner {
    pub fn add_pack_instructions(
        block: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        memory: MemoryId,
    ) {
        IAddress::add_pack_instructions(block, module, local, writer_pointer, memory)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::{Address, U256, address};
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::packing::Packable,
        test_compilation_context,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };

    fn test_uint(int_type: impl Packable, data: &[u8], expected_result: &[u8]) {
        let (mut raw_module, alloc_function, memory_id) = build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);
        let compilation_ctx = test_compilation_context!(memory_id, alloc_function);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Mock literal allocation (is already in memory)
        func_body.i32_const(data.len() as i32);
        func_body.call(alloc_function);
        func_body.local_set(local);

        func_body.i32_const(int_type.encoded_size(&compilation_ctx) as i32);
        func_body.call(alloc_function);
        func_body.local_set(writer_pointer);

        // Args data should already be stored in memory
        int_type.add_pack_instructions(
            &mut func_body,
            &mut raw_module,
            local,
            writer_pointer,
            writer_pointer, // unused for this type
            &compilation_ctx,
        );

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        // the return is the pointer to the packed value
        let result: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result);
    }

    #[test]
    fn test_pack_u128() {
        type IntType = u128;
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IU128;

        let expected_result = SolType::abi_encode_params(&(128128128128,));
        test_uint(
            int_type.clone(),
            &128128128128u128.to_le_bytes(),
            &expected_result,
        );

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(
            int_type.clone(),
            &IntType::MAX.to_le_bytes(),
            &expected_result,
        ); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(
            int_type.clone(),
            &IntType::MIN.to_le_bytes(),
            &expected_result,
        ); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - 1,));
        test_uint(
            int_type.clone(),
            &(IntType::MAX - 1).to_le_bytes(),
            &expected_result,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_pack_u256() {
        type IntType = U256;
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IU256;

        let expected_result = SolType::abi_encode_params(&(U256::from(256256256256u128),));
        test_uint(
            int_type.clone(),
            &U256::from(256256256256u128).to_le_bytes::<32>(),
            &expected_result,
        );

        let expected_result = SolType::abi_encode_params(&(IntType::MAX,));
        test_uint(
            int_type.clone(),
            &IntType::MAX.to_le_bytes::<32>(),
            &expected_result,
        ); // max

        let expected_result = SolType::abi_encode_params(&(IntType::MIN,));
        test_uint(
            int_type.clone(),
            &IntType::MIN.to_le_bytes::<32>(),
            &expected_result,
        ); // min

        let expected_result = SolType::abi_encode_params(&(IntType::MAX - U256::from(1),));
        test_uint(
            int_type.clone(),
            &(IntType::MAX - U256::from(1)).to_le_bytes::<32>(),
            &expected_result,
        ); // max -1 (avoid symmetry)
    }

    #[test]
    fn test_pack_address() {
        type SolType = sol!((address,));
        let int_type = IntermediateType::IAddress;

        let expected_result = SolType::abi_encode_params(&(Address::ZERO,));
        test_uint(int_type.clone(), &expected_result, &expected_result);

        let expected_result =
            SolType::abi_encode_params(&(address!("0x1234567890abcdef1234567890abcdef12345678"),));
        test_uint(int_type.clone(), &expected_result, &expected_result);

        let expected_result =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        test_uint(int_type.clone(), &expected_result, &expected_result);

        let expected_result =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE"),));
        test_uint(int_type.clone(), &expected_result, &expected_result);
    }

    #[test]
    fn test_pack_signer() {
        type SolType = sol!((address,));
        let int_type = IntermediateType::ISigner;

        let expected_result = SolType::abi_encode_params(&(Address::ZERO,));
        test_uint(int_type.clone(), &expected_result, &expected_result);

        let expected_result =
            SolType::abi_encode_params(&(address!("0x1234567890abcdef1234567890abcdef12345678"),));
        test_uint(int_type.clone(), &expected_result, &expected_result);

        let expected_result =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        test_uint(int_type.clone(), &expected_result, &expected_result);

        let expected_result =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE"),));
        test_uint(int_type.clone(), &expected_result, &expected_result);
    }
}
