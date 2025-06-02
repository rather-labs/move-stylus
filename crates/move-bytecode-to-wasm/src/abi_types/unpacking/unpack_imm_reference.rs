use super::Unpackable;
use crate::translation::intermediate_types::IntermediateType;
use crate::translation::intermediate_types::imm_reference::IRef;
use walrus::{
    FunctionId, InstrSeqBuilder, LocalId, MemoryId, Module,
    ir::{MemArg, StoreKind},
};

impl IRef {
    pub fn add_unpack_instructions(
        inner: &IntermediateType,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        reader_pointer: LocalId,
        calldata_reader_pointer: LocalId,
        memory: MemoryId,
        allocator: FunctionId,
    ) {
        match inner {
            // If inner is a heap type, forward the pointer
            IntermediateType::IVector(_)
            | IntermediateType::IAddress
            | IntermediateType::ISigner
            | IntermediateType::IU128
            | IntermediateType::IU256 => {
                inner.add_unpack_instructions(
                    builder,
                    module,
                    reader_pointer,
                    calldata_reader_pointer,
                    memory,
                    allocator,
                );
            }
            // For immediates, allocate and store
            IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IBool => {
                let ptr_local = module.locals.add(walrus::ValType::I32);

                builder.i32_const(inner.stack_data_size() as i32);
                builder.call(allocator);
                builder.local_tee(ptr_local);

                inner.add_unpack_instructions(
                    builder,
                    module,
                    reader_pointer,
                    calldata_reader_pointer,
                    memory,
                    allocator,
                );

                builder.store(
                    memory,
                    match inner.stack_data_size() {
                        4 => StoreKind::I32 { atomic: false },
                        8 => StoreKind::I64 { atomic: false },
                        _ => panic!("Unsupported stack_data_size for IRef"),
                    },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );

                builder.local_get(ptr_local);
            }

            IntermediateType::IRef(_) => {
                panic!("Inner type cannot be a reference!");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::IntermediateType,
    };
    use alloy_primitives::{U256, address};
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    fn test_unpack_ref(data: &[u8], ref_type: IntermediateType, expected_memory_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();
        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(0);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        ref_type.add_unpack_instructions(
            &mut func_body,
            &mut raw_module,
            args_pointer,
            calldata_reader_pointer,
            memory_id,
            allocator,
        );

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();
        // result_ptr is always 0, is that ok?

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_memory_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(
            result_memory_data, expected_memory_bytes,
            "Heap memory at returned pointer does not match expected content"
        );
    }

    #[test]
    fn test_unpack_ref_u8() {
        type SolType = sol!((uint8,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU8));

        let data = SolType::abi_encode_params(&(88u8,));

        let mut expected = Vec::new();
        expected.extend(&88u32.to_le_bytes());

        test_unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u32() {
        type SolType = sol!((uint32,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU32));

        let data = SolType::abi_encode_params(&(88u32,));
        test_unpack_ref(&data, int_type.clone(), &88u32.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u64() {
        type SolType = sol!((uint64,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU64));

        let data = SolType::abi_encode_params(&(88u64,));
        test_unpack_ref(&data, int_type.clone(), &88u64.to_le_bytes());
    }

    #[test]
    fn test_unpack_ref_u128() {
        type SolType = sol!((uint128,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU128));

        let data = SolType::abi_encode_params(&(123u128,));
        let expected = 123u128.to_le_bytes().to_vec();
        test_unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_u256() {
        type SolType = sol!((uint256,));
        let int_type = IntermediateType::IRef(Box::new(IntermediateType::IU256));

        let value = U256::from(123u128);
        let expected = value.to_le_bytes::<32>().to_vec();

        let data = SolType::abi_encode_params(&(value,));
        test_unpack_ref(&data, int_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_vec_u8() {
        type SolType = sol!((uint8[],));
        let inner_type = IntermediateType::IU8;
        let vector_type = IntermediateType::IRef(Box::new(IntermediateType::IVector(Box::new(
            inner_type.clone(),
        ))));

        let vec_data = vec![1u8, 2u8, 3u8];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&(vec_data.len() as u32).to_le_bytes()); // length inside header
        for v in &vec_data {
            expected.extend(&(*v as u32).to_le_bytes()); // pad each u8 to i32 (4 bytes)
        }

        test_unpack_ref(&data, vector_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_vec_128() {
        type SolType = sol!((uint128[],));
        let inner_type = IntermediateType::IU128;
        let vector_type = IntermediateType::IRef(Box::new(IntermediateType::IVector(Box::new(
            inner_type.clone(),
        ))));

        let vec_data = vec![1u128, 2u128];
        let data = SolType::abi_encode_params(&(vec_data.clone(),));

        let mut expected = Vec::new();
        expected.extend(&(vec_data.len() as u32).to_le_bytes()); // length = 2

        // pointers to heap elements
        expected.extend(&12u32.to_le_bytes());
        expected.extend(&28u32.to_le_bytes());

        // first u128 at 16
        expected.extend(&1u128.to_le_bytes());

        // second u128 at 32
        expected.extend(&2u128.to_le_bytes());

        test_unpack_ref(&data, vector_type.clone(), &expected);
    }

    #[test]
    fn test_unpack_ref_address() {
        type SolType = sol!((address,));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IAddress));

        let data =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        test_unpack_ref(&data, ref_type.clone(), &data);
    }
}
