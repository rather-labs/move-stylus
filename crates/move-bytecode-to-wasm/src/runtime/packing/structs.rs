use crate::{
    CompilationContext,
    abi_types::packing::Packable,
    data::RuntimeErrorData,
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::{
        IntermediateType,
        structs::{IStruct, IStructType},
    },
};
use std::collections::HashMap;
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg},
};

/// Generates a WASM function that packs a Move struct into Solidity ABI tuple format.
///
/// The function handles both static and dynamic structs. For static structs, fields are packed
/// directly at the writer_pointer. For dynamic structs (when is_nested=1), memory is allocated
/// and an offset is written at writer_pointer. Fields with dynamic types (vectors, dynamic structs)
/// are packed with offsets to their actual data locations.
///
/// For event structs, indexed fields are excluded from the packed data as they are handled separately.
///
/// # WASM Function Arguments:
/// * `struct_pointer` (i32) - pointer to the Move struct structure
/// * `writer_pointer` (i32) - pointer where the packed struct should be written
/// * `calldata_reference_pointer` (i32) - reference point for calculating relative offsets
/// * `is_nested` (i32) - flag indicating if this is a nested dynamic struct (1) or root struct (0)
///
/// # WASM Function Returns:
/// * None - the result is written directly to memory at writer_pointer or at allocated memory
pub fn pack_struct_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::PackStruct.get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[],
    );

    let mut builder: walrus::InstrSeqBuilder<'_> = function.name(name).func_body();

    // If the struct is an event we need to exclue the indexed fields as those are not part of the data.
    // Else we use the original struct.
    let struct_ = {
        let struct_ = compilation_ctx.get_struct_by_intermediate_type(itype)?;
        if let IStructType::Event { indexes, .. } = struct_.type_ {
            IStruct::new(
                move_binary_format::file_format::StructDefinitionIndex(0),
                &format!("{}Data", struct_.identifier),
                struct_.fields[indexes as usize..]
                    .iter()
                    .map(|t| (None, t.clone()))
                    .collect(),
                HashMap::new(),
                false,
                IStructType::Common,
            )
        } else {
            struct_.into_owned()
        }
    };

    // Arguments
    let struct_pointer = module.locals.add(ValType::I32);
    let writer_pointer = module.locals.add(ValType::I32);
    let calldata_reference_pointer = module.locals.add(ValType::I32);
    let is_nested = module.locals.add(ValType::I32);

    let val_32 = module.locals.add(ValType::I32);
    let val_64 = module.locals.add(ValType::I64);
    let reference_value = module.locals.add(ValType::I32);

    let data_ptr = module.locals.add(ValType::I32);
    let inner_data_reference = module.locals.add(ValType::I32);

    // Compute the size before the closure since closures that return () cannot use ?
    let struct_size = struct_.solidity_abi_encode_size(compilation_ctx)? as i32;
    let pack_u32_function = RuntimeFunction::PackU32.get(module, Some(compilation_ctx))?;

    // If is_nested is 1, means we are packing an struct inside a struct and that the struct is dynamic.
    builder.local_get(is_nested).if_else(
        None,
        |then| {
            // Allocate memory for the packed value. Set the data_ptr the beginning, since
            // we are going to pack the values from there
            then.i32_const(struct_size)
                .call(compilation_ctx.allocator)
                .local_tee(data_ptr)
                .local_tee(inner_data_reference);

            // The pointer in the packed data must be relative to the calldata_reference_pointer,
            // so we substract calldata_reference_pointer from the writer_pointer
            then.local_get(calldata_reference_pointer)
                .binop(BinaryOp::I32Sub)
                .local_set(reference_value);

            // The result is saved where calldata_reference_pointer is pointing at, the value will
            // be the address where the struct  values are packed, using as origin
            // calldata_reference_pointer
            then.local_get(reference_value)
                .local_get(writer_pointer)
                .call(pack_u32_function);
        },
        |else_| {
            else_.local_get(writer_pointer).local_set(data_ptr);
        },
    );

    // Load the value to be written in the calldata, if it is a stack value we need to double
    // reference a pointer, otherwise we read the pointer and leave the stack value in the
    // stack
    for (index, field) in struct_.fields.iter().enumerate() {
        // Load field's intermediate pointer
        builder.local_get(struct_pointer).load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: index as u32 * 4,
            },
        );

        // Load the value
        let field_local = match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let val = match ValType::try_from(field)? {
                    ValType::I64 => val_64,
                    _ => val_32,
                };

                builder
                    .load(
                        compilation_ctx.memory_id,
                        field.load_kind()?,
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(val);

                val
            }
            _ => {
                builder.local_set(val_32);
                val_32
            }
        };

        // If is_nested == 0, means we are not packing this struct
        // dynamically, so, we can set inner_data_reference as the root reference pointer
        builder.block(None, |block| {
            let block_id = block.id();
            block.local_get(is_nested).br_if(block_id);

            block
                .local_get(calldata_reference_pointer)
                .local_set(inner_data_reference);
        });

        // If the field to pack is a struct, it will be packed dynamically, that means, in the
        // current offset of writer pointer, we are going to write the offset where we can find
        // the struct
        let advancement: Result<usize, RuntimeFunctionError> = match field {
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. } => {
                let child_struct = compilation_ctx.get_struct_by_intermediate_type(field)?;

                if child_struct.solidity_abi_encode_is_dynamic(compilation_ctx)? {
                    field.add_pack_instructions_dynamic(
                        &mut builder,
                        module,
                        field_local,
                        data_ptr,
                        inner_data_reference,
                        compilation_ctx,
                        Some(runtime_error_data),
                    )?;
                    Ok(32)
                } else {
                    field.add_pack_instructions(
                        &mut builder,
                        module,
                        field_local,
                        data_ptr,
                        inner_data_reference,
                        compilation_ctx,
                        Some(runtime_error_data),
                    )?;
                    Ok(field.encoded_size(compilation_ctx)?)
                }
            }
            _ => {
                field.add_pack_instructions(
                    &mut builder,
                    module,
                    field_local,
                    data_ptr,
                    inner_data_reference,
                    compilation_ctx,
                    Some(runtime_error_data),
                )?;
                Ok(32)
            }
        };

        // The value of advacement depends on the following conditions:
        // - If the field we are encoding is a static struct, the pointer must be advanced the size
        //   of the tuple that represents the struct.
        // - If the field we are encoding is a dynamic struct, we just need to advance the pointer
        //   32 bytes because in the argument's place there is only a pointer to where the
        //   struct's values are packed
        // - If it is not a struct:
        //   - If it is a static field it will occupy 32 bytes,
        //   - if it is a dynamic field, the offset pointing to where to find the values will be
        //     written, also occuping 32 bytes.
        let advancement = advancement?;
        builder
            .i32_const(advancement as i32)
            .local_get(data_ptr)
            .binop(BinaryOp::I32Add)
            .local_set(data_ptr);
    }

    Ok(function.finish(
        vec![
            struct_pointer,
            writer_pointer,
            calldata_reference_pointer,
            is_nested,
        ],
        &mut module.funcs,
    ))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::compilation_context::ModuleData;
    use alloy_primitives::{U256, address};
    use alloy_sol_types::{SolValue, sol};
    use rstest::rstest;
    use std::cell::RefCell;
    use std::panic::AssertUnwindSafe;
    use std::rc::Rc;
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::packing::Packable,
        compilation_context::ModuleId,
        test_compilation_context, test_runtime_error_data,
        test_tools::{build_module, setup_wasmtime_module},
        translation::intermediate_types::{
            IntermediateType, VmHandledStruct,
            structs::{IStruct, IStructType},
        },
    };

    #[rstest]
    #[case::struct_u32_bool(
        vec![
            IntermediateType::IU32,
            IntermediateType::IBool,
        ],
        [
            8u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),
            42u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    uint32 x;
                    bool y;
                }
            }
            TestStruct { x: 42u32, y: true }.abi_encode()
        }
    )]
    #[case::struct_u8_u16_u64(
        vec![
            IntermediateType::IU8,
            IntermediateType::IU16,
            IntermediateType::IU64,
        ],
        [
            12u32.to_le_bytes().as_slice(),
            13u32.to_le_bytes().as_slice(),
            15u32.to_le_bytes().as_slice(),
            10u8.to_le_bytes().as_slice(),
            20u16.to_le_bytes().as_slice(),
            30u64.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    uint8 a;
                    uint16 b;
                    uint64 c;
                }
            }
            TestStruct { a: 10u8, b: 20u16, c: 30u64 }.abi_encode()
        }
    )]
    #[case::struct_bool_u32_bool(
        vec![
            IntermediateType::IBool,
            IntermediateType::IU32,
            IntermediateType::IBool,
        ],
        [
            12u32.to_le_bytes().as_slice(),
            13u32.to_le_bytes().as_slice(),
            17u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            123u32.to_le_bytes().as_slice(),
            0u8.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    bool a;
                    uint32 b;
                    bool c;
                }
            }
            TestStruct { a: true, b: 123u32, c: false }.abi_encode()
        }
    )]
    #[case::struct_u32_u128(
        vec![
            IntermediateType::IU32,
            IntermediateType::IU128,
        ],
        [
            8u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),
            100u32.to_le_bytes().as_slice(),
            123456789u128.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    uint32 x;
                    uint128 y;
                }
            }
            TestStruct { x: 100u32, y: 123456789u128 }.abi_encode()
        }
    )]
    #[case::struct_u256_bool(
        vec![
            IntermediateType::IU256,
            IntermediateType::IBool,
        ],
        [
            8u32.to_le_bytes().as_slice(),
            40u32.to_le_bytes().as_slice(),
            U256::from(999u64).to_le_bytes::<32>().as_slice(),
            1u8.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    uint256 x;
                    bool y;
                }
            }
            TestStruct { x: U256::from(999u64), y: true }.abi_encode()
        }
    )]
    #[case::struct_address_u32(
        vec![
            IntermediateType::IAddress,
            IntermediateType::IU32,
        ],
        [
            8u32.to_le_bytes().as_slice(),
            40u32.to_le_bytes().as_slice(),
            [&[0; 12], address!("0x1234567890abcdef1234567890abcdef12345678").as_slice()].concat().as_slice(),
            777u32.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    address addr;
                    uint32 val;
                }
            }
            TestStruct {
                addr: address!("0x1234567890abcdef1234567890abcdef12345678"),
                val: 777u32
            }.abi_encode()
        }
    )]
    #[case::struct_four_u32(
        vec![
            IntermediateType::IU32,
            IntermediateType::IU32,
            IntermediateType::IU32,
            IntermediateType::IU32,
        ],
        [
            16u32.to_le_bytes().as_slice(),
            20u32.to_le_bytes().as_slice(),
            24u32.to_le_bytes().as_slice(),
            28u32.to_le_bytes().as_slice(),
            1u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            4u32.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    uint32 a;
                    uint32 b;
                    uint32 c;
                    uint32 d;
                }
            }
            TestStruct { a: 1u32, b: 2u32, c: 3u32, d: 4u32 }.abi_encode()
        }
    )]
    #[case::struct_u64_u8_u16(
        vec![
            IntermediateType::IU64,
            IntermediateType::IU8,
            IntermediateType::IU16,
        ],
        [
            12u32.to_le_bytes().as_slice(),
            20u32.to_le_bytes().as_slice(),
            21u32.to_le_bytes().as_slice(),
            9876543210u64.to_le_bytes().as_slice(),
            5u8.to_le_bytes().as_slice(),
            300u16.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    uint64 a;
                    uint8 b;
                    uint16 c;
                }
            }
            TestStruct { a: 9876543210u64, b: 5u8, c: 300u16 }.abi_encode()
        }
    )]
    #[case::struct_u32_vec_u8(
        vec![
            IntermediateType::IU32,
            IntermediateType::IVector(Arc::new(IntermediateType::IU8)),
        ],
        [
            8u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),
            500u32.to_le_bytes().as_slice(),
            // Vector: len, capacity, elements
            3u32.to_le_bytes().as_slice(),
            3u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            2u8.to_le_bytes().as_slice(),
            3u8.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    uint32 a;
                    uint8[] b;
                }
            }
            TestStruct { a: 500u32, b: vec![1u8, 2u8, 3u8] }.abi_encode_sequence()
        }
    )]
    #[case::struct_bool_vec_u64(
        vec![
            IntermediateType::IBool,
            IntermediateType::IVector(Arc::new(IntermediateType::IU64)),
        ],
        [
            8u32.to_le_bytes().as_slice(),
            9u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
            // Vector: len, capacity, elements
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            100u64.to_le_bytes().as_slice(),
            200u64.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    bool a;
                    uint64[] b;
                }
            }
            TestStruct { a: true, b: vec![100u64, 200u64] }.abi_encode_sequence()
        }
    )]
    #[case::struct_u16_vec_u128(
        vec![
            IntermediateType::IU16,
            IntermediateType::IVector(Arc::new(IntermediateType::IU128)),
        ],
        [
            8u32.to_le_bytes().as_slice(),
            10u32.to_le_bytes().as_slice(),
            999u16.to_le_bytes().as_slice(),
            // Vector: len, capacity, pointer array, then u128 values
            2u32.to_le_bytes().as_slice(),
            2u32.to_le_bytes().as_slice(),
            26u32.to_le_bytes().as_slice(), // pointer to first u128
            42u32.to_le_bytes().as_slice(), // pointer to second u128
            111111u128.to_le_bytes().as_slice(),
            222222u128.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct TestStruct {
                    uint16 a;
                    uint128[] b;
                }
            }
            TestStruct { a: 999u16, b: vec![111111u128, 222222u128] }.abi_encode_sequence()
        }
    )]
    #[case::struct_with_substruct(
        vec![
            IntermediateType::IU32,
            IntermediateType::IStruct {
                module_id: ModuleId::default(),
                index: 1,
                vm_handled_struct: VmHandledStruct::None,
            },
        ],
        [
            8u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),
            42u32.to_le_bytes().as_slice(),
            // SubStruct pointer array
            20u32.to_le_bytes().as_slice(),
            24u32.to_le_bytes().as_slice(),
            // SubStruct field values
            10u32.to_le_bytes().as_slice(),
            1u8.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct SubStruct {
                    uint32 x;
                    bool y;
                }
                struct TestStruct {
                    uint32 a;
                    SubStruct b;
                }
            }
            TestStruct {
                a: 42u32,
                b: SubStruct { x: 10u32, y: true }
            }.abi_encode_sequence()
        }
    )]
    #[case::struct_with_substruct_vec_u128(
        vec![
            IntermediateType::IU32,
            IntermediateType::IStruct {
                module_id: ModuleId::default(),
                index: 2,
                vm_handled_struct: VmHandledStruct::None,
            },
        ],
        [
            8u32.to_le_bytes().as_slice(),
            12u32.to_le_bytes().as_slice(),
            100u32.to_le_bytes().as_slice(),
            // SubStruct pointer array (bool and vec<u128>)
            20u32.to_le_bytes().as_slice(),
            21u32.to_le_bytes().as_slice(),
            // SubStruct field values
            1u8.to_le_bytes().as_slice(), // bool value
            // Vector: len, capacity, pointer array for u128, then u128 values
            3u32.to_le_bytes().as_slice(), // length
            3u32.to_le_bytes().as_slice(), // capacity
            41u32.to_le_bytes().as_slice(), // pointer to first u128
            57u32.to_le_bytes().as_slice(), // pointer to second u128
            73u32.to_le_bytes().as_slice(), // pointer to third u128
            111u128.to_le_bytes().as_slice(),
            222u128.to_le_bytes().as_slice(),
            333u128.to_le_bytes().as_slice(),
        ].concat(),
        {
            sol! {
                struct SubStructWithVec {
                    bool x;
                    uint128[] y;
                }
                struct TestStruct {
                    uint32 a;
                    SubStructWithVec b;
                }
            }
            TestStruct {
                a: 100u32,
                b: SubStructWithVec { x: true, y: vec![111u128, 222u128, 333u128] }
            }.abi_encode_sequence()
        }
    )]
    fn test_struct_packing(
        #[case] fields: Vec<IntermediateType>,
        #[case] data: Vec<u8>,
        #[case] expected_result: Vec<u8>,
    ) {
        use crate::compilation_context::ModuleData;

        let (mut raw_module, alloc_function, memory_id, ctx_globals) = build_module(None);

        let mut compilation_ctx = test_compilation_context!(memory_id, alloc_function, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let struct_type = IntermediateType::IStruct {
            module_id: ModuleId::default(),
            index: 0,
            vm_handled_struct: VmHandledStruct::None,
        };

        let test_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(0),
            "TestStruct",
            fields.iter().map(|f| (None, f.clone())).collect(),
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let sub_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(1),
            "SubStruct",
            vec![
                (None, IntermediateType::IU32),
                (None, IntermediateType::IBool),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let sub_struct_with_vec = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(2),
            "SubStructWithVec",
            vec![
                (None, IntermediateType::IBool),
                (
                    None,
                    IntermediateType::IVector(Arc::new(IntermediateType::IU128)),
                ),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let mut module_data = ModuleData::default();

        let module_structs = vec![test_struct, sub_struct, sub_struct_with_vec];
        module_data.structs.structs = module_structs;

        compilation_ctx.root_module_data = &module_data;

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();

        let struct_pointer = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(data.len() as i32);
        func_body.call(alloc_function);
        func_body.local_set(struct_pointer);

        func_body.i32_const(struct_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(alloc_function);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        struct_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                struct_pointer,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, data.to_vec(), "test_function", None);

        let result: i32 = entrypoint.call(&mut store, ()).unwrap();

        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_memory_data = vec![0; expected_result.len()];
        memory
            .read(&mut store, result as usize, &mut result_memory_data)
            .unwrap();
        assert_eq!(result_memory_data, expected_result);
    }

    #[test]
    fn test_pack_struct_mixed_static_types_fuzz() {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);

        let mut compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let struct_type = IntermediateType::IStruct {
            module_id: ModuleId::default(),
            index: 0,
            vm_handled_struct: VmHandledStruct::None,
        };

        let test_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(0),
            "TestStruct",
            vec![
                (None, IntermediateType::IU8),
                (None, IntermediateType::IU64),
                (None, IntermediateType::IU128),
                (None, IntermediateType::IBool),
                (None, IntermediateType::IAddress),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let mut module_data = ModuleData::default();
        module_data.structs.structs = vec![test_struct];
        compilation_ctx.root_module_data = &module_data;

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let struct_pointer = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(struct_pointer);

        func_body.i32_const(struct_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        struct_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                struct_pointer,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<(u8, u64, u128, bool, [u8; 20])>()
            .cloned()
            .for_each(
                |(a, b, c, d, addr_bytes): (u8, u64, u128, bool, [u8; 20])| {
                    // Build memory layout: pointer array, then values
                    let mut data = vec![];
                    // Pointer array (5 fields)
                    data.extend(&20u32.to_le_bytes()); // u8 at offset 20
                    data.extend(&21u32.to_le_bytes()); // u64 at offset 21
                    data.extend(&29u32.to_le_bytes()); // u128 at offset 29 (pointer)
                    data.extend(&45u32.to_le_bytes()); // bool at offset 45
                    data.extend(&46u32.to_le_bytes()); // address at offset 46 (pointer)
                    // Values
                    data.extend(&[a]); // u8 value
                    data.extend(&b.to_le_bytes()); // u64 value
                    data.extend(&c.to_le_bytes()); // u128 value
                    data.extend(&[d as u8]); // bool value
                    data.extend(&[0u8; 12]); // address padding
                    data.extend(&addr_bytes); // address value

                    memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                    let result_ptr: i32 = entrypoint
                        .0
                        .call(&mut *store.0.borrow_mut(), data.len() as i32)
                        .unwrap();

                    sol! {
                        struct TestStruct {
                            uint8 a;
                            uint64 b;
                            uint128 c;
                            bool d;
                            address e;
                        }
                    }
                    let addr = alloy_primitives::Address::from_slice(&addr_bytes);
                    let expected = TestStruct {
                        a,
                        b,
                        c,
                        d,
                        e: addr,
                    }
                    .abi_encode();
                    let mut result_memory_data = vec![0; expected.len()];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            result_ptr as usize,
                            &mut result_memory_data,
                        )
                        .unwrap();

                    assert_eq!(
                        result_memory_data, expected,
                        "Packed struct with mixed types did not match expected result",
                    );

                    reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
                },
            );
    }

    // Test dynamic types inside sturcts
    #[test]
    fn test_pack_struct_with_vectors_fuzz() {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);

        let mut compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let struct_type = IntermediateType::IStruct {
            module_id: ModuleId::default(),
            index: 0,
            vm_handled_struct: VmHandledStruct::None,
        };

        let test_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(0),
            "TestStruct",
            vec![
                (None, IntermediateType::IU64),
                (None, IntermediateType::IU128),
                (None, IntermediateType::IBool),
                (
                    None,
                    IntermediateType::IVector(Arc::new(IntermediateType::IU128)),
                ),
                (
                    None,
                    IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
                ),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let mut module_data = ModuleData::default();
        module_data.structs.structs = vec![test_struct];
        compilation_ctx.root_module_data = &module_data;

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let struct_pointer = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(struct_pointer);

        func_body.i32_const(struct_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        struct_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                struct_pointer,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<(u64, u128, bool, Vec<u128>, Vec<u32>)>()
            .cloned()
            .for_each(
                |(a, b, c, vec_u128, vec_u32): (u64, u128, bool, Vec<u128>, Vec<u32>)| {
                    let len_u128 = vec_u128.len() as u32;
                    let len_u32 = vec_u32.len() as u32;

                    // Calculate offsets
                    let struct_ptr_array_size = 20; // 5 fields * 4 bytes
                    let a_offset = struct_ptr_array_size;
                    let b_offset = a_offset + 8; // u64 size
                    let c_offset = b_offset + 16; // u128 size
                    let vec_u128_offset = c_offset + 1; // bool size
                    let vec_u128_ptr_array_offset = vec_u128_offset + 8; // vec header (len + cap)
                    let vec_u128_data_offset = vec_u128_ptr_array_offset + (len_u128 as usize * 4);
                    let vec_u32_offset = vec_u128_data_offset + (len_u128 as usize * 16);

                    // Build memory layout
                    let mut data = vec![];
                    // Struct pointer array
                    data.extend(&(a_offset as u32).to_le_bytes());
                    data.extend(&(b_offset as u32).to_le_bytes());
                    data.extend(&(c_offset as u32).to_le_bytes());
                    data.extend(&(vec_u128_offset as u32).to_le_bytes());
                    data.extend(&(vec_u32_offset as u32).to_le_bytes());

                    // Values
                    data.extend(&a.to_le_bytes()); // u64
                    data.extend(&b.to_le_bytes()); // u128
                    data.extend(&[c as u8]); // bool

                    // vec<u128>: len, cap, pointer array, values
                    data.extend(&len_u128.to_le_bytes());
                    data.extend(&len_u128.to_le_bytes());
                    for i in 0..len_u128 {
                        let ptr = (vec_u128_data_offset + (i as usize * 16)) as u32;
                        data.extend(&ptr.to_le_bytes());
                    }
                    for val in &vec_u128 {
                        data.extend(&val.to_le_bytes());
                    }

                    // vec<u32>: len, cap, values
                    data.extend(&len_u32.to_le_bytes());
                    data.extend(&len_u32.to_le_bytes());
                    for val in &vec_u32 {
                        data.extend(&val.to_le_bytes());
                    }

                    memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                    let result_ptr: i32 = entrypoint
                        .0
                        .call(&mut *store.0.borrow_mut(), data.len() as i32)
                        .unwrap();

                    sol! {
                        struct TestStruct {
                            uint64 a;
                            uint128 b;
                            bool c;
                            uint128[] d;
                            uint32[] e;
                        }
                    }
                    let expected = TestStruct {
                        a,
                        b,
                        c,
                        d: vec_u128.clone(),
                        e: vec_u32.clone(),
                    }
                    .abi_encode_sequence();
                    let mut result_memory_data = vec![0; expected.len()];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            result_ptr as usize,
                            &mut result_memory_data,
                        )
                        .unwrap();

                    assert_eq!(
                        result_memory_data, expected,
                        "Packed struct with vectors did not match expected result",
                    );

                    reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
                },
            );
    }

    // Substruct without dynamic types
    #[test]
    fn test_pack_struct_with_simple_substruct_fuzz() {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);

        let mut compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let struct_type = IntermediateType::IStruct {
            module_id: ModuleId::default(),
            index: 0,
            vm_handled_struct: VmHandledStruct::None,
        };

        let test_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(0),
            "TestStruct",
            vec![
                (None, IntermediateType::IU64),
                (None, IntermediateType::IU128),
                (
                    None,
                    IntermediateType::IStruct {
                        module_id: ModuleId::default(),
                        index: 1,
                        vm_handled_struct: VmHandledStruct::None,
                    },
                ),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let sub_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(1),
            "SubStruct",
            vec![
                (None, IntermediateType::IBool),
                (None, IntermediateType::IAddress),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let mut module_data = ModuleData::default();
        module_data.structs.structs = vec![test_struct, sub_struct];
        compilation_ctx.root_module_data = &module_data;

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let struct_pointer = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(struct_pointer);

        func_body.i32_const(struct_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        struct_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                struct_pointer,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<(u64, u128, bool, [u8; 20])>()
            .cloned()
            .for_each(
                |(a, b, sub_bool, addr_bytes): (u64, u128, bool, [u8; 20])| {
                    // Build memory layout
                    let mut data = vec![];

                    // Main struct pointer array (3 fields)
                    data.extend(&12u32.to_le_bytes()); // u64 at offset 12
                    data.extend(&20u32.to_le_bytes()); // u128 at offset 20
                    data.extend(&36u32.to_le_bytes()); // substruct at offset 36

                    // Main struct values
                    data.extend(&a.to_le_bytes()); // u64
                    data.extend(&b.to_le_bytes()); // u128

                    // SubStruct pointer array (2 fields)
                    data.extend(&44u32.to_le_bytes()); // bool at offset 44
                    data.extend(&45u32.to_le_bytes()); // address at offset 45

                    // SubStruct values
                    data.extend(&[sub_bool as u8]); // bool
                    data.extend(&[0u8; 12]); // address padding
                    data.extend(&addr_bytes); // address

                    memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                    let result_ptr: i32 = entrypoint
                        .0
                        .call(&mut *store.0.borrow_mut(), data.len() as i32)
                        .unwrap();

                    sol! {
                        struct SubStruct {
                            bool x;
                            address y;
                        }
                        struct TestStruct {
                            uint64 a;
                            uint128 b;
                            SubStruct c;
                        }
                    }
                    let addr = alloy_primitives::Address::from_slice(&addr_bytes);
                    let expected = TestStruct {
                        a,
                        b,
                        c: SubStruct {
                            x: sub_bool,
                            y: addr,
                        },
                    }
                    .abi_encode_sequence();
                    let mut result_memory_data = vec![0; expected.len()];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            result_ptr as usize,
                            &mut result_memory_data,
                        )
                        .unwrap();

                    assert_eq!(
                        result_memory_data, expected,
                        "Packed struct with simple substruct did not match expected result",
                    );

                    reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
                },
            );
    }

    #[test]
    fn test_pack_struct_with_dynamic_substruct_fuzz() {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(None);

        let mut compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = test_runtime_error_data!();

        let struct_type = IntermediateType::IStruct {
            module_id: ModuleId::default(),
            index: 0,
            vm_handled_struct: VmHandledStruct::None,
        };

        let test_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(0),
            "TestStruct",
            vec![
                (None, IntermediateType::IU64),
                (None, IntermediateType::IU128),
                (
                    None,
                    IntermediateType::IStruct {
                        module_id: ModuleId::default(),
                        index: 1,
                        vm_handled_struct: VmHandledStruct::None,
                    },
                ),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let sub_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(1),
            "SubStruct",
            vec![
                (
                    None,
                    IntermediateType::IVector(Arc::new(IntermediateType::IU128)),
                ),
                (
                    None,
                    IntermediateType::IVector(Arc::new(IntermediateType::IU32)),
                ),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let mut module_data = ModuleData::default();
        module_data.structs.structs = vec![test_struct, sub_struct];
        compilation_ctx.root_module_data = &module_data;

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[ValType::I32], &[ValType::I32]);

        let mut func_body = function_builder.func_body();

        let data_space = raw_module.locals.add(ValType::I32);
        let struct_pointer = raw_module.locals.add(ValType::I32);
        let writer_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reference_pointer = raw_module.locals.add(ValType::I32);

        func_body.local_get(data_space);
        func_body.call(allocator);
        func_body.local_set(struct_pointer);

        func_body.i32_const(struct_type.encoded_size(&compilation_ctx).unwrap() as i32);
        func_body.call(allocator);
        func_body.local_tee(writer_pointer);
        func_body.local_set(calldata_reference_pointer);

        struct_type
            .add_pack_instructions(
                &mut func_body,
                &mut raw_module,
                struct_pointer,
                writer_pointer,
                calldata_reference_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
            )
            .unwrap();

        func_body.local_get(writer_pointer);

        let function = function_builder.finish(vec![data_space], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module::<i32, i32>(&mut raw_module, vec![], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<(u64, u128, Vec<u128>, Vec<u32>)>()
            .cloned()
            .for_each(
                |(a, b, vec_u128, vec_u32): (u64, u128, Vec<u128>, Vec<u32>)| {
                    let len_u128 = vec_u128.len() as u32;
                    let len_u32 = vec_u32.len() as u32;

                    // Calculate offsets
                    let main_ptr_array_size = 12; // 3 fields * 4 bytes
                    let a_offset = main_ptr_array_size;
                    let b_offset = a_offset + 8; // u64 size
                    let substruct_offset = b_offset + 16; // u128 size
                    let substruct_ptr_array_size = 8; // 2 fields * 4 bytes
                    let vec_u128_offset = substruct_offset + substruct_ptr_array_size;
                    let vec_u128_ptr_array_offset = vec_u128_offset + 8; // vec header
                    let vec_u128_data_offset = vec_u128_ptr_array_offset + (len_u128 as usize * 4);
                    let vec_u32_offset = vec_u128_data_offset + (len_u128 as usize * 16);

                    // Build memory layout
                    let mut data = vec![];

                    // Main struct pointer array
                    data.extend(&(a_offset as u32).to_le_bytes());
                    data.extend(&(b_offset as u32).to_le_bytes());
                    data.extend(&(substruct_offset as u32).to_le_bytes());

                    // Main struct values
                    data.extend(&a.to_le_bytes()); // u64
                    data.extend(&b.to_le_bytes()); // u128

                    // SubStruct pointer array
                    data.extend(&(vec_u128_offset as u32).to_le_bytes());
                    data.extend(&(vec_u32_offset as u32).to_le_bytes());

                    // vec<u128>: len, cap, pointer array, values
                    data.extend(&len_u128.to_le_bytes());
                    data.extend(&len_u128.to_le_bytes());
                    for i in 0..len_u128 {
                        let ptr = (vec_u128_data_offset + (i as usize * 16)) as u32;
                        data.extend(&ptr.to_le_bytes());
                    }
                    for val in &vec_u128 {
                        data.extend(&val.to_le_bytes());
                    }

                    // vec<u32>: len, cap, values
                    data.extend(&len_u32.to_le_bytes());
                    data.extend(&len_u32.to_le_bytes());
                    for val in &vec_u32 {
                        data.extend(&val.to_le_bytes());
                    }

                    memory.write(&mut *store.0.borrow_mut(), 0, &data).unwrap();

                    let result_ptr: i32 = entrypoint
                        .0
                        .call(&mut *store.0.borrow_mut(), data.len() as i32)
                        .unwrap();

                    sol! {
                        struct SubStruct {
                            uint128[] x;
                            uint32[] y;
                        }
                        struct TestStruct {
                            uint64 a;
                            uint128 b;
                            SubStruct c;
                        }
                    }
                    let expected = TestStruct {
                        a,
                        b,
                        c: SubStruct {
                            x: vec_u128.clone(),
                            y: vec_u32.clone(),
                        },
                    }
                    .abi_encode_sequence();
                    let mut result_memory_data = vec![0; expected.len()];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            result_ptr as usize,
                            &mut result_memory_data,
                        )
                        .unwrap();

                    assert_eq!(
                        result_memory_data, expected,
                        "Packed struct with dynamic substruct did not match expected result",
                    );

                    reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
                },
            );
    }
}
