use super::Packable;
use crate::CompilationContext;
use crate::translation::intermediate_types::IntermediateType;
use crate::translation::intermediate_types::reference::{IMutRef, IRef};
use walrus::{
    InstrSeqBuilder, LocalId, Module, ValType,
    ir::{LoadKind, MemArg},
};

impl IRef {
    #[allow(clippy::too_many_arguments)]
    pub fn add_pack_instructions(
        inner: &IntermediateType,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        match inner {
            // Heap types: just forward the pointer
            IntermediateType::IVector(_)
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _)
            | IntermediateType::ISigner
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress => {
                inner.add_pack_instructions(
                    builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );
            }
            // Immediate types: deref the pointer and pass the value as LocalId
            IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IBool => {
                builder.local_get(local);
                builder.load(
                    compilation_ctx.memory_id,
                    match inner.stack_data_size() {
                        4 => LoadKind::I32 { atomic: false },
                        8 => LoadKind::I64 { atomic: false },
                        _ => panic!("Unsupported stack_data_size for IRef pack"),
                    },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                let value_local = module.locals.add(match inner.stack_data_size() {
                    4 => ValType::I32,
                    8 => ValType::I64,
                    _ => panic!("Unsupported stack_data_size for IRef pack"),
                });
                builder.local_set(value_local);

                inner.add_pack_instructions(
                    builder,
                    module,
                    value_local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );
            }
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("Inner type cannot be a reference!");
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot pack generic type parameter");
            }
            IntermediateType::IEnum(_) => todo!(),
            IntermediateType::IExternalUserData { .. } => todo!(),
        }
    }
}

impl IMutRef {
    #[allow(clippy::too_many_arguments)]
    pub fn add_pack_instructions(
        inner: &IntermediateType,
        builder: &mut InstrSeqBuilder,
        module: &mut Module,
        local: LocalId,
        writer_pointer: LocalId,
        calldata_reference_pointer: LocalId,
        compilation_ctx: &CompilationContext,
    ) {
        match inner {
            // Heap types: just forward the pointer
            IntermediateType::IVector(_)
            | IntermediateType::ISigner
            | IntermediateType::IU128
            | IntermediateType::IU256
            | IntermediateType::IAddress
            | IntermediateType::IStruct(_)
            | IntermediateType::IGenericStructInstance(_, _) => {
                inner.add_pack_instructions(
                    builder,
                    module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );
            }
            // Immediate types: deref the pointer and pass the value as LocalId
            IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64
            | IntermediateType::IBool => {
                builder.local_get(local);
                builder.load(
                    compilation_ctx.memory_id,
                    match inner.stack_data_size() {
                        4 => LoadKind::I32 { atomic: false },
                        8 => LoadKind::I64 { atomic: false },
                        _ => panic!("Unsupported stack_data_size for IRef pack"),
                    },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
                let value_local = module.locals.add(match inner.stack_data_size() {
                    4 => ValType::I32,
                    8 => ValType::I64,
                    _ => panic!("Unsupported stack_data_size for IRef pack"),
                });
                builder.local_set(value_local);

                inner.add_pack_instructions(
                    builder,
                    module,
                    value_local,
                    writer_pointer,
                    calldata_reference_pointer,
                    compilation_ctx,
                );
            }
            IntermediateType::IEnum(_) => todo!(),
            IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
                panic!("Inner type cannot be a reference!");
            }
            IntermediateType::ITypeParameter(_) => {
                panic!("cannot pack generic type parameter");
            }
            IntermediateType::IExternalUserData { .. } => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_compilation_context;
    use crate::test_tools::build_module;
    use crate::test_tools::setup_wasmtime_module;
    use crate::translation::intermediate_types::IntermediateType;
    use alloy_primitives::address;
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    fn test_pack(data: &[u8], ref_type: IntermediateType, expected_calldata_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id) = build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx = test_compilation_context!(memory_id, allocator);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate data (what to write)
        func_body.i32_const(data.len() as i32);
        func_body.call(allocator);
        func_body.local_set(local);

        // Allocate calldata (where to write)
        func_body.i32_const(ref_type.encoded_size(&compilation_ctx) as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        // Pack the data to calldata memory
        ref_type.add_pack_instructions(
            &mut func_body,
            &mut raw_module,
            local,
            writer_pointer,
            calldata_reference_pointer,
            &compilation_ctx,
        );

        // Return the writer pointer for reading the calldata back
        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result_ptr: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_calldata_bytes.len()];
        memory
            .read(&mut store, result_ptr as usize, &mut result_memory_data)
            .unwrap();

        assert_eq!(
            result_memory_data, expected_calldata_bytes,
            "Packed calldata did not match expected result"
        );
    }

    #[test]
    fn test_pack_ref_u8() {
        type SolType = sol!((uint8,));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IU8));
        let heap_data = 88u32.to_le_bytes().to_vec();
        let expected = SolType::abi_encode_params(&(88u8,));
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_u32() {
        type SolType = sol!((uint32,));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IU32));
        let heap_data = 88u32.to_le_bytes().to_vec();
        let expected = SolType::abi_encode_params(&(88u32,));
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_u64() {
        type SolType = sol!((uint64,));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IU64));
        let heap_data = 88u64.to_le_bytes().to_vec();
        let expected = SolType::abi_encode_params(&(88u64,));
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_u128() {
        type SolType = sol!((uint128,));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IU128));
        let heap_data = 88u128.to_le_bytes().to_vec();
        let expected = SolType::abi_encode_params(&(88u128,));
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_address() {
        type SolType = sol!((address,));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IAddress));
        let expected =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        test_pack(&expected, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_signer() {
        type SolType = sol!((address,));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::ISigner));

        let expected_result =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        test_pack(&expected_result, ref_type.clone(), &expected_result);
    }

    #[test]
    fn test_pack_ref_vec_u8() {
        type SolType = sol!((uint8[],));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU8,
        ))));

        let expected = SolType::abi_encode_params(&(vec![1u8, 2u8, 3u8],));

        test_pack(
            &[
                3u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                1u32.to_le_bytes().as_slice(),
                2u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
            ]
            .concat(),
            ref_type.clone(),
            &expected,
        );
    }

    #[test]
    fn test_pack_ref_vec_u128() {
        type SolType = sol!((uint128[],));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IU128,
        ))));

        let mut heap_data = Vec::new();

        // 1. Length = 3
        heap_data.extend(&3u32.to_le_bytes());
        heap_data.extend(&4u32.to_le_bytes());

        // 2. Pointers to heap-allocated u128 values
        heap_data.extend(&24u32.to_le_bytes());
        heap_data.extend(&40u32.to_le_bytes());
        heap_data.extend(&56u32.to_le_bytes());
        heap_data.extend(&0u32.to_le_bytes());

        // 3. Actual values at those pointers (u128 little endian)
        heap_data.extend(&1u128.to_le_bytes());
        heap_data.extend(&2u128.to_le_bytes());
        heap_data.extend(&3u128.to_le_bytes());

        // Expected ABI calldata after packing (flat vector encoding)
        let expected_calldata = SolType::abi_encode_params(&(vec![1u128, 2u128, 3u128],));

        test_pack(&heap_data, ref_type.clone(), &expected_calldata);
    }

    #[test]
    fn test_pack_ref_vector_vector_u32() {
        type SolType = sol!((uint32[][],));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IVector(Box::new(IntermediateType::IU32)),
        ))));

        let expected_result = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));

        let data = [
            2u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),  // capacity
            24u32.to_le_bytes().as_slice(), // pointer to first element
            56u32.to_le_bytes().as_slice(), // pointer to second element
            0u32.to_le_bytes().as_slice(),  // first buffer mem
            0u32.to_le_bytes().as_slice(),  // second buffer mem
            3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
            5u32.to_le_bytes().as_slice(),
            6u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
            0u32.to_le_bytes().as_slice(),
        ]
        .concat();
        test_pack(&data, ref_type.clone(), &expected_result);
    }

    #[test]
    fn test_pack_ref_vector_vector_u128() {
        type SolType = sol!((uint128[][],));
        let ref_type = IntermediateType::IRef(Box::new(IntermediateType::IVector(Box::new(
            IntermediateType::IVector(Box::new(IntermediateType::IU128)),
        ))));

        let expected_result = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));
        let data = [
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            16u32.to_le_bytes().as_slice(),
            84u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            36u32.to_le_bytes().as_slice(),
            52u32.to_le_bytes().as_slice(),
            68u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            104u32.to_le_bytes().as_slice(),
            120u32.to_le_bytes().as_slice(),
            136u32.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
            5u128.to_le_bytes().as_slice(),
            6u128.to_le_bytes().as_slice(),
        ]
        .concat();
        test_pack(&data, ref_type.clone(), &expected_result);
    }
}
