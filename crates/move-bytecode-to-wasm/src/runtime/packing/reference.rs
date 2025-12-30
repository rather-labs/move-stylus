use crate::{
    CompilationContext,
    abi_types::error::{AbiError, AbiOperationError},
    abi_types::packing::Packable,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{LoadKind, MemArg},
};

pub fn pack_reference_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    inner: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name =
        RuntimeFunction::PackReference.get_generic_function_name(compilation_ctx, &[inner])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );

    let mut builder: walrus::InstrSeqBuilder<'_> = function.name(name).func_body();

    // Arguments
    let reference_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);

    match inner {
        IntermediateType::ISigner
        | IntermediateType::IU128
        | IntermediateType::IU256
        | IntermediateType::IAddress
        | IntermediateType::IVector(_)
        | IntermediateType::IStruct { .. }
        | IntermediateType::IGenericStructInstance { .. }
        | IntermediateType::IEnum { .. }
        | IntermediateType::IGenericEnumInstance { .. } => {
            // Load the intermediate pointer and pack
            builder
                .local_get(reference_pointer)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_set(reference_pointer);

            inner.add_pack_instructions(
                &mut builder,
                module,
                reference_pointer,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            )?;
        }
        IntermediateType::IBool
        | IntermediateType::IU8
        | IntermediateType::IU16
        | IntermediateType::IU32
        | IntermediateType::IU64 => {
            // Load the intermediate pointer
            builder
                .local_get(reference_pointer)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
                .local_tee(reference_pointer);

            builder.load(
                compilation_ctx.memory_id,
                inner.load_kind()?,
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            let value_local = module.locals.add(ValType::try_from(inner)?);
            builder.local_set(value_local);

            inner.add_pack_instructions(
                &mut builder,
                module,
                value_local,
                writer_pointer,
                calldata_reference_pointer,
                compilation_ctx,
            )?;
        }
        IntermediateType::IRef(_) | IntermediateType::IMutRef(_) => {
            Err(AbiError::Pack(AbiOperationError::RefInsideRef))?;
        }
        IntermediateType::ITypeParameter(_) => {
            Err(AbiError::Pack(
                AbiOperationError::PackingGenericTypeParameter,
            ))?;
        }
    }

    Ok(function.finish(
        vec![
            reference_pointer,
            writer_pointer,
            calldata_reference_pointer,
        ],
        &mut module.funcs,
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::test_compilation_context;
    use crate::test_tools::build_module;
    use crate::test_tools::setup_wasmtime_module;
    use crate::translation::intermediate_types::IntermediateType;
    use alloy_primitives::address;
    use alloy_sol_types::{SolType, sol};
    use walrus::{FunctionBuilder, ValType};

    fn test_pack(data: &[u8], ref_type: IntermediateType, expected_calldata_bytes: &[u8]) {
        let (mut raw_module, allocator, memory_id, calldata_reader_pointer_global) =
            build_module(None);

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let compilation_ctx =
            test_compilation_context!(memory_id, allocator, calldata_reader_pointer_global);

        let local = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        let mut func_body = function_builder.func_body();

        // Allocate data (what to write)
        func_body.i32_const(data.len() as i32);
        func_body.call(allocator);
        func_body.local_set(local);

        // Allocate calldata (where to write)
        func_body.i32_const(ref_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        if ref_type.is_dynamic(&compilation_ctx).unwrap() {
            ref_type
                .add_pack_instructions_dynamic(
                    &mut func_body,
                    &mut raw_module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    &compilation_ctx,
                )
                .unwrap();
        } else {
            ref_type
                .add_pack_instructions(
                    &mut func_body,
                    &mut raw_module,
                    local,
                    writer_pointer,
                    calldata_reference_pointer,
                    &compilation_ctx,
                )
                .unwrap();
        };

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
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU8));
        let mut heap_data = Vec::new();
        heap_data.extend(&4u32.to_le_bytes()); // Pointer to the u8 data
        heap_data.extend(&88u8.to_le_bytes()); // Actual u8 data
        let expected = SolType::abi_encode_params(&(88u8,));
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_u32() {
        type SolType = sol!((uint32,));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU32));
        let mut heap_data = Vec::new();
        heap_data.extend(&4u32.to_le_bytes()); // Pointer to the u32 data
        heap_data.extend(&88u32.to_le_bytes()); // Actual u32 data
        let expected = SolType::abi_encode_params(&(88u32,));
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_u64() {
        type SolType = sol!((uint64,));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU64));
        let mut heap_data = Vec::new();
        heap_data.extend(&4u32.to_le_bytes()); // Pointer to the u64 data
        heap_data.extend(&88u64.to_le_bytes()); // Actual u64 data
        let expected = SolType::abi_encode_params(&(88u64,));
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_u128() {
        type SolType = sol!((uint128,));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IU128));
        let mut heap_data = Vec::new();
        heap_data.extend(&4u32.to_le_bytes()); // Pointer to the u128 data
        heap_data.extend(&88u128.to_le_bytes()); // Actual u128 data
        let expected = SolType::abi_encode_params(&(88u128,));
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_address() {
        type SolType = sol!((address,));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IAddress));
        let mut heap_data = Vec::new();
        let expected =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        heap_data.extend(&4u32.to_le_bytes()); // Pointer to the address data
        heap_data.extend(&expected); // Actual address data
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    #[should_panic]
    fn test_pack_ref_signer() {
        type SolType = sol!((address,));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::ISigner));

        let mut heap_data = Vec::new();
        let expected =
            SolType::abi_encode_params(&(address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),));
        heap_data.extend(&4u32.to_le_bytes()); // Pointer to the address data
        heap_data.extend(&expected); // Actual address data
        test_pack(&heap_data, ref_type.clone(), &expected);
    }

    #[test]
    fn test_pack_ref_vec_u8() {
        type SolType = sol!((uint8[],));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU8,
        ))));

        let expected = SolType::abi_encode_params(&(vec![1u8, 2u8, 3u8],));

        test_pack(
            &[
                4u32.to_le_bytes().as_slice(), // pointer to the vector
                3u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                1u8.to_le_bytes().as_slice(),
                2u8.to_le_bytes().as_slice(),
                3u8.to_le_bytes().as_slice(),
            ]
            .concat(),
            ref_type.clone(),
            &expected,
        );
    }

    #[test]
    fn test_pack_ref_vec_u16() {
        type SolType = sol!((uint16[],));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU16,
        ))));

        let expected = SolType::abi_encode_params(&(vec![1u16, 2u16, 3u16],));

        test_pack(
            &[
                4u32.to_le_bytes().as_slice(), // pointer to the vector
                3u32.to_le_bytes().as_slice(),
                3u32.to_le_bytes().as_slice(),
                1u16.to_le_bytes().as_slice(),
                2u16.to_le_bytes().as_slice(),
                3u16.to_le_bytes().as_slice(),
            ]
            .concat(),
            ref_type.clone(),
            &expected,
        );
    }

    #[test]
    fn test_pack_ref_vec_u32() {
        type SolType = sol!((uint32[],));
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU32,
        ))));

        let expected = SolType::abi_encode_params(&(vec![1u32, 2u32, 3u32],));

        test_pack(
            &[
                4u32.to_le_bytes().as_slice(), // pointer to the vector
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
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IU128,
        ))));

        let mut heap_data = Vec::new();
        heap_data.extend(&4u32.to_le_bytes()); // pointer to the vector

        // 1. Length = 3
        heap_data.extend(&3u32.to_le_bytes());
        heap_data.extend(&4u32.to_le_bytes());

        // 2. Pointers to heap-allocated u128 values
        heap_data.extend(&28u32.to_le_bytes());
        heap_data.extend(&44u32.to_le_bytes());
        heap_data.extend(&60u32.to_le_bytes());
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
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
        ))));

        let expected_result = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));

        let data = [
            4u32.to_le_bytes().as_slice(), // pointer to the vector
            2u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),  // capacity
            28u32.to_le_bytes().as_slice(), // pointer to first element
            60u32.to_le_bytes().as_slice(), // pointer to second element
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
        let ref_type = IntermediateType::IRef(Arc::new(IntermediateType::IVector(Arc::new(
            IntermediateType::IVector(Arc::new(IntermediateType::IU128)),
        ))));

        let expected_result = SolType::abi_encode_params(&(vec![vec![1, 2, 3], vec![4, 5, 6]],));
        let data = [
            4u32.to_le_bytes().as_slice(), // pointer to the vector
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            20u32.to_le_bytes().as_slice(),
            88u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            40u32.to_le_bytes().as_slice(),
            56u32.to_le_bytes().as_slice(),
            72u32.to_le_bytes().as_slice(),
            1u128.to_le_bytes().as_slice(),
            2u128.to_le_bytes().as_slice(),
            3u128.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            108u32.to_le_bytes().as_slice(),
            124u32.to_le_bytes().as_slice(),
            140u32.to_le_bytes().as_slice(),
            4u128.to_le_bytes().as_slice(),
            5u128.to_le_bytes().as_slice(),
            6u128.to_le_bytes().as_slice(),
        ]
        .concat();
        test_pack(&data, ref_type.clone(), &expected_result);
    }
}
