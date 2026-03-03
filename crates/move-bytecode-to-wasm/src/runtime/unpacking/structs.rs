use crate::{
    CompilationContext,
    abi_types::unpacking::{ObjectKind, Unpackable},
    data::{DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, RuntimeErrorData},
    runtime::{RuntimeFunction, RuntimeFunctionError},
    translation::intermediate_types::IntermediateType,
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

// According to the formal specification of the encoding, a tuple (T1,...,Tk) is dynamic if
// Ti is dynamic for some 1 <= i <= k.
//
// Since structs are encoded as tuples of their fields, a struct is also considered dynamic
// if any of its fields is dynamic.
//
// Based on the ABI specification, the following types are considered dynamic:
// - bytes
// - string
// - T[] for any T
// - T[k] for any dynamic T and any k >= 0
// - (T1,...,Tk) if Ti is dynamic for some 1 <= i <= k
//
// For example, the following Move's struct:
//
// public struct Foo has drop {
//    x: u8,
//    y: vector<u32>,
//    z: vector<u128>,
// }
//
// Is equivalent to the following struct in Solidity:
//
// struct Foo {
//     uint8 x;
//     uint32[] y;
//     uint128[] z;
// }
//
// Given that the struct contains vectors, it becomes a dynamic. This means that the first encoded
// value of this struct will be a number pointing to where the values are packed in the calldata.
//
// If we call a function that have Foo as an argument with:
// Foo {
//     x: 254,
//     y: [1, 2, u32::MAX],
//     z: [1, 2, u128::MAX],
// }
//
// The encoded data will be:
// bytes   0..3      4..35   36..67   68..99   100..131
//       [selector,   32  ,   254   ,   96   ,   224  , [3,1,2,u32::MAX], [3,1,2,u128::MAX]]
//                 ptr_foo  ▲  x       ptr_y    ptr_z   ▲                 ▲
//                    │     │           │         │     │                 │
//                    └─────┘           └─────────┼─────┘                 │
//                                                └───────────────────────┘
// where
//  - selector: the called function selector
//
//  - ptr_foo: where the Foo struct's values are packed in the calldata. It is 32 because it does
//    not take in account the selector.  36 = len(selector) + len(ptr_foo) = 4 + 32,
//    where the packed data starts
//
//  - x: 254 packed as uint8 (32 bytes)
//
//  - ptr_y: where the y's vector values are packed. It does not take in account the selector and
//    ptr_foo. 96 = len(x) + len(ptr_y) + len(ptr_z) = 32 + 32 + 32
//
//  - ptr_z: where the z's vector values are packed. It does not take in account the selector and
//    ptr_foo. 224 = len(x) + len(ptr_y) + len(ptr_z) + y_data = 32 + 32 + 32 + 128.
//    y_data has length 128 because it contains its length (32 bytes) and 3 elements (3 x 32bytes)
//
// If a struct does not contain any dynamic fields, all its fields are encoded inline, packed
// contiguously without any offset or pointer.
//
// For more information:
// https://docs.soliditylang.org/en/develop/abi-spec.html#formal-specification-of-the-encoding
///
/// Generates a runtime function that unpacks a Move struct from ABI-encoded calldata.
///
/// The function supports both static and dynamic ABI struct layouts, unpacks each field
/// recursively, stores field pointers in a newly allocated Move struct representation, and
/// advances the calldata reader pointer according to the struct ABI shape.
///
/// # WASM Function Arguments
/// * `reader_pointer` - (i32): pointer to the struct argument head in calldata
/// * `calldata_reader_pointer` - (i32): base pointer used to resolve dynamic field offsets
///
/// # WASM Function Returns
/// * `struct_ptr` - (i32): pointer to the allocated unpacked Move struct
pub fn unpack_struct_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name =
        RuntimeFunction::UnpackStruct.get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    // Arguments
    let reader_pointer = module.locals.add(ValType::I32);
    let calldata_reader_pointer = module.locals.add(ValType::I32);

    // Moving pointer for reading data of the fields
    let data_reader_pointer = module.locals.add(ValType::I32);

    // Pointer to where the struct is packed
    let calldata_ptr = module.locals.add(ValType::I32);

    let struct_ = compilation_ctx.get_struct_by_intermediate_type(itype)?;

    // In a dynamic struct, the first value is where the values are packed in the calldata
    if struct_.solidity_abi_encode_is_dynamic(compilation_ctx)? {
        // Big-endian to Little-endian
        let swap_i32_bytes_function = RuntimeFunction::SwapI32Bytes.get(module, None, None)?;

        // Validate that the pointer fits in 32 bits
        let validate_pointer_fn = RuntimeFunction::ValidatePointer32Bit.get(
            module,
            Some(compilation_ctx),
            Some(runtime_error_data),
        )?;
        builder.local_get(reader_pointer).call_runtime_function(
            compilation_ctx,
            validate_pointer_fn,
            &RuntimeFunction::ValidatePointer32Bit,
            Some(ValType::I32),
        );

        builder
            .local_get(reader_pointer)
            .load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    // Abi encoded value is Big endian
                    offset: 28,
                },
            )
            .call(swap_i32_bytes_function)
            .local_get(calldata_reader_pointer)
            .binop(BinaryOp::I32Add)
            .local_tee(data_reader_pointer)
            .local_set(calldata_ptr);
    } else {
        builder
            .local_get(reader_pointer)
            .local_set(data_reader_pointer)
            .local_get(calldata_reader_pointer)
            .local_set(calldata_ptr);
    }

    // Allocate space for the struct
    let struct_ptr = module.locals.add(ValType::I32);
    builder
        .i32_const(struct_.heap_size as i32)
        .call(compilation_ctx.allocator)
        .local_set(struct_ptr);

    let mut offset = 0;
    let field_ptr = module.locals.add(ValType::I32);
    for field in &struct_.fields {
        // Unpack field
        field.add_unpack_instructions(
            Some(itype),
            &mut builder,
            module,
            None,
            Some(ValType::I32),
            data_reader_pointer,
            calldata_ptr,
            compilation_ctx,
            Some(runtime_error_data),
            None,
        )?;

        // If the field is stack type, we need to create the intermediate pointer, otherwise
        // the add_unpack_instructions function leaves the pointer in the stack
        match field {
            IntermediateType::IBool
            | IntermediateType::IU8
            | IntermediateType::IU16
            | IntermediateType::IU32
            | IntermediateType::IU64 => {
                let data_size = field.wasm_memory_data_size()?;
                let val = module.locals.add(ValType::try_from(field)?);
                let store_kind = field.store_kind()?;

                // Save the actual value
                builder.local_set(val);

                // Create a pointer for the value
                builder
                    .i32_const(data_size)
                    .call(compilation_ctx.allocator)
                    .local_tee(field_ptr);

                // Store the actual value behind the middle_ptr
                builder.local_get(val).store(
                    compilation_ctx.memory_id,
                    store_kind,
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                );
            }
            _ => {
                builder.local_set(field_ptr);
            }
        }

        builder.local_get(struct_ptr).local_get(field_ptr).store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg { align: 0, offset },
        );

        offset += 4;
    }

    // Advance reader pointer after processing struct.
    // If it is a static struct, the pointer must be advanced the size of the tuple that
    // represents the struct.
    // If it is a dynamic struct, we just need to advance the pointer 32 bytes because in the
    // argument's place there is only a pointer to where the values of the struct are packed
    let advancement = if struct_.solidity_abi_encode_is_dynamic(compilation_ctx)? {
        32
    } else {
        struct_.solidity_abi_encode_size(compilation_ctx)? as i32
    };

    builder
        .local_get(reader_pointer)
        .i32_const(advancement)
        .binop(BinaryOp::I32Add)
        .global_set(compilation_ctx.globals.calldata_reader_pointer);

    builder.local_get(struct_ptr);

    Ok(function.finish(
        vec![reader_pointer, calldata_reader_pointer],
        &mut module.funcs,
    ))
}

/// Unpacks a storage struct by locating it in the appropriate storage mapping and decoding it.
///
/// The `object_kind` parameter is resolved at compilation time, so only the code for the
/// specific storage mapping is generated. This saves gas compared to runtime branching.
///
/// When `object_kind` is `Some(kind)`:
///   - The wasm function takes a single parameter: `uid_ptr` (i32)
///   - It directly calls the corresponding locate function (owned / shared / frozen)
///
/// When `object_kind` is `None`:
///   - The wasm function takes two parameters: `uid_ptr` (i32) and `unpack_frozen` (i32)
///   - It calls `LocateStorageData(uid_ptr, unpack_frozen)` which searches multiple mappings
///
/// The function name includes the object kind as a suffix to distinguish different variants.
///
/// # WASM Function Arguments
/// * `uid_ptr` - (i32): pointer to the object UID used to locate storage data
/// * `unpack_frozen` - (i32): flag used only when `object_kind` is `None`
///
/// # WASM Function Returns
/// * `struct_ptr` - (i32): pointer to the decoded storage struct in Move memory
pub fn unpack_storage_struct_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
    object_kind: Option<ObjectKind>,
) -> Result<FunctionId, RuntimeFunctionError> {
    let base_name = RuntimeFunction::UnpackStorageStruct
        .get_generic_function_name(compilation_ctx, &[itype])?;
    let suffix = match object_kind {
        Some(ObjectKind::Owned) => "_owned",
        Some(ObjectKind::Shared) => "_shared",
        Some(ObjectKind::Frozen) => "_frozen",
        None => "",
    };
    let name = format!("{base_name}{suffix}");
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    // When the object kind is known at compile time, we only need uid_ptr.
    // When unknown, we also need unpack_frozen to search multiple storage mappings.
    let params: Vec<ValType> = if object_kind.is_some() {
        vec![ValType::I32]
    } else {
        vec![ValType::I32, ValType::I32]
    };

    let mut function = FunctionBuilder::new(&mut module.types, &params, &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    // Arguments
    let uid_ptr = module.locals.add(ValType::I32);

    // Resolve and call the appropriate locate_storage function based on object_kind.
    // The match also returns param_locals so that `unpack_frozen` can be declared as a
    // plain LocalId inside the `None` arm — the only place it exists.
    let param_locals = match object_kind {
        Some(ObjectKind::Owned) => {
            let locate_fn = RuntimeFunction::LocateStorageOwnedData.get(
                module,
                Some(compilation_ctx),
                Some(runtime_error_data),
            )?;
            builder.local_get(uid_ptr).call_runtime_function(
                compilation_ctx,
                locate_fn,
                &RuntimeFunction::LocateStorageOwnedData,
                Some(ValType::I32),
            );
            vec![uid_ptr]
        }
        Some(ObjectKind::Shared) => {
            let locate_fn = RuntimeFunction::LocateStorageSharedData.get(
                module,
                Some(compilation_ctx),
                Some(runtime_error_data),
            )?;
            builder.local_get(uid_ptr).call_runtime_function(
                compilation_ctx,
                locate_fn,
                &RuntimeFunction::LocateStorageSharedData,
                Some(ValType::I32),
            );
            vec![uid_ptr]
        }
        Some(ObjectKind::Frozen) => {
            let locate_fn = RuntimeFunction::LocateStorageFrozenData.get(
                module,
                Some(compilation_ctx),
                Some(runtime_error_data),
            )?;
            builder.local_get(uid_ptr).call_runtime_function(
                compilation_ctx,
                locate_fn,
                &RuntimeFunction::LocateStorageFrozenData,
                Some(ValType::I32),
            );
            vec![uid_ptr]
        }
        None => {
            let unpack_frozen = module.locals.add(ValType::I32);
            let locate_fn = RuntimeFunction::LocateStorageData.get(
                module,
                Some(compilation_ctx),
                Some(runtime_error_data),
            )?;
            builder
                .local_get(uid_ptr)
                .local_get(unpack_frozen)
                .call_runtime_function(
                    compilation_ctx,
                    locate_fn,
                    &RuntimeFunction::LocateStorageData,
                    Some(ValType::I32),
                );
            vec![uid_ptr, unpack_frozen]
        }
    };

    let owner_id_ptr = module.locals.add(ValType::I32);
    builder.local_set(owner_id_ptr);

    // Read the object
    let read_and_decode_from_storage_fn = RuntimeFunction::ReadAndDecodeFromStorage.get_generic(
        module,
        compilation_ctx,
        Some(runtime_error_data),
        &[itype],
    )?;

    // Copy the slot number into a local to avoid overwriting it later
    let slot_ptr = module.locals.add(ValType::I32);
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_tee(slot_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    builder
        .local_get(slot_ptr)
        .local_get(uid_ptr)
        .local_get(owner_id_ptr)
        .call(read_and_decode_from_storage_fn);

    Ok(function.finish(param_locals, &mut module.funcs))
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap, panic::AssertUnwindSafe, rc::Rc, sync::Arc};

    use alloy_primitives::Address;
    use alloy_sol_types::{SolValue, sol};
    use walrus::{FunctionBuilder, ValType};

    use crate::{
        abi_types::unpacking::Unpackable,
        compilation_context::{ModuleData, ModuleId},
        data::RuntimeErrorData,
        test_compilation_context,
        test_tools::{INITIAL_MEMORY_OFFSET, build_module, setup_wasmtime_module},
        translation::intermediate_types::{
            IntermediateType, VmHandledStruct,
            structs::{IStruct, IStructType},
        },
    };

    #[test]
    fn test_unpack_struct_mixed_static_types_fuzz() {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(4096));

        let mut compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

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
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();

        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        struct_type
            .add_unpack_instructions(
                Some(&struct_type),
                &mut func_body,
                &mut raw_module,
                None,
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 4096], "test_function", None);

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
                    sol! {
                        struct TestStruct {
                            uint8 a;
                            uint64 b;
                            uint128 c;
                            bool d;
                            address e;
                        }
                    }
                    let addr = Address::from_slice(&addr_bytes);
                    let data = TestStruct {
                        a,
                        b,
                        c,
                        d,
                        e: addr,
                    }
                    .abi_encode();

                    if data.len() > 4096 {
                        return;
                    }

                    memory
                        .write(
                            &mut *store.0.borrow_mut(),
                            INITIAL_MEMORY_OFFSET as usize,
                            &data,
                        )
                        .unwrap();

                    let result_ptr: i32 =
                        entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                    // Read the struct pointer array (5 pointers)
                    let mut struct_data = vec![0; 20];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            result_ptr as usize,
                            &mut struct_data,
                        )
                        .unwrap();

                    // Read each field through its pointer
                    let u8_ptr = u32::from_le_bytes([
                        struct_data[0],
                        struct_data[1],
                        struct_data[2],
                        struct_data[3],
                    ]) as usize;
                    let u64_ptr = u32::from_le_bytes([
                        struct_data[4],
                        struct_data[5],
                        struct_data[6],
                        struct_data[7],
                    ]) as usize;
                    let u128_ptr = u32::from_le_bytes([
                        struct_data[8],
                        struct_data[9],
                        struct_data[10],
                        struct_data[11],
                    ]) as usize;
                    let bool_ptr = u32::from_le_bytes([
                        struct_data[12],
                        struct_data[13],
                        struct_data[14],
                        struct_data[15],
                    ]) as usize;
                    let addr_ptr = u32::from_le_bytes([
                        struct_data[16],
                        struct_data[17],
                        struct_data[18],
                        struct_data[19],
                    ]) as usize;

                    let mut u8_data = [0u8; 1];
                    memory
                        .read(&mut *store.0.borrow_mut(), u8_ptr, &mut u8_data)
                        .unwrap();
                    assert_eq!(u8_data[0], a, "u8 field mismatch");

                    let mut u64_data = [0u8; 8];
                    memory
                        .read(&mut *store.0.borrow_mut(), u64_ptr, &mut u64_data)
                        .unwrap();
                    assert_eq!(u64::from_le_bytes(u64_data), b, "u64 field mismatch");

                    let mut u128_data = [0u8; 16];
                    memory
                        .read(&mut *store.0.borrow_mut(), u128_ptr, &mut u128_data)
                        .unwrap();
                    assert_eq!(u128::from_le_bytes(u128_data), c, "u128 field mismatch");

                    let mut bool_data = [0u8; 1];
                    memory
                        .read(&mut *store.0.borrow_mut(), bool_ptr, &mut bool_data)
                        .unwrap();
                    assert_eq!(bool_data[0], d as u8, "bool field mismatch");

                    let mut addr_data = [0u8; 32];
                    memory
                        .read(&mut *store.0.borrow_mut(), addr_ptr, &mut addr_data)
                        .unwrap();
                    assert_eq!(
                        Address::from_slice(&addr_data[12..32]),
                        addr,
                        "address field mismatch"
                    );

                    reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
                },
            );
    }

    #[test]
    fn test_unpack_struct_with_vectors_fuzz() {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(16384));

        let mut compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

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
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();

        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        struct_type
            .add_unpack_instructions(
                Some(&struct_type),
                &mut func_body,
                &mut raw_module,
                None,
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 8192], "test_function", None);

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
                    sol! {
                        struct TestStruct {
                            uint64 a;
                            uint128 b;
                            bool c;
                            uint128[] d;
                            uint32[] e;
                        }
                    }
                    let struct_ = TestStruct {
                        a,
                        b,
                        c,
                        d: vec_u128.clone(),
                        e: vec_u32.clone(),
                    };
                    let data = struct_.abi_encode();

                    memory
                        .write(
                            &mut *store.0.borrow_mut(),
                            INITIAL_MEMORY_OFFSET as usize,
                            &data,
                        )
                        .unwrap();

                    let result_ptr: i32 =
                        entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                    // Read the struct pointer array (5 pointers)
                    let mut struct_data = vec![0; 20];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            result_ptr as usize,
                            &mut struct_data,
                        )
                        .unwrap();

                    // Verify scalar fields
                    let u64_ptr = u32::from_le_bytes([
                        struct_data[0],
                        struct_data[1],
                        struct_data[2],
                        struct_data[3],
                    ]) as usize;
                    let u128_ptr = u32::from_le_bytes([
                        struct_data[4],
                        struct_data[5],
                        struct_data[6],
                        struct_data[7],
                    ]) as usize;
                    let bool_ptr = u32::from_le_bytes([
                        struct_data[8],
                        struct_data[9],
                        struct_data[10],
                        struct_data[11],
                    ]) as usize;
                    let vec_u128_ptr = u32::from_le_bytes([
                        struct_data[12],
                        struct_data[13],
                        struct_data[14],
                        struct_data[15],
                    ]) as usize;
                    let vec_u32_ptr = u32::from_le_bytes([
                        struct_data[16],
                        struct_data[17],
                        struct_data[18],
                        struct_data[19],
                    ]) as usize;

                    let mut u64_data = [0u8; 8];
                    memory
                        .read(&mut *store.0.borrow_mut(), u64_ptr, &mut u64_data)
                        .unwrap();

                    assert_eq!(u64::from_le_bytes(u64_data), a, "u64 field mismatch");

                    let mut u128_data = [0u8; 16];
                    memory
                        .read(&mut *store.0.borrow_mut(), u128_ptr, &mut u128_data)
                        .unwrap();

                    assert_eq!(u128::from_le_bytes(u128_data), b, "u128 field mismatch");

                    let mut bool_data = [0u8; 1];
                    memory
                        .read(&mut *store.0.borrow_mut(), bool_ptr, &mut bool_data)
                        .unwrap();

                    // Verify vec<u128>
                    let mut vec_u128_header = [0u8; 8];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            vec_u128_ptr,
                            &mut vec_u128_header,
                        )
                        .unwrap();
                    let vec_u128_len = u32::from_le_bytes([
                        vec_u128_header[0],
                        vec_u128_header[1],
                        vec_u128_header[2],
                        vec_u128_header[3],
                    ]) as usize;
                    assert_eq!(vec_u128_len, vec_u128.len(), "vec<u128> length mismatch");

                    // For heap types (u128), the vector stores pointers
                    for (i, &expected_val) in vec_u128.iter().enumerate() {
                        let ptr_offset = vec_u128_ptr + 8 + (i * 4);
                        let mut ptr_bytes = [0u8; 4];
                        memory
                            .read(&mut *store.0.borrow_mut(), ptr_offset, &mut ptr_bytes)
                            .unwrap();
                        let element_ptr = u32::from_le_bytes(ptr_bytes) as usize;

                        let mut element_data = [0u8; 16];
                        memory
                            .read(&mut *store.0.borrow_mut(), element_ptr, &mut element_data)
                            .unwrap();

                        let stored_val = u128::from_le_bytes(element_data);
                        assert_eq!(stored_val, expected_val, "vec<u128>[{i}] mismatch");
                    }

                    // Verify vec<u32>
                    let mut vec_u32_header = [0u8; 8];
                    memory
                        .read(&mut *store.0.borrow_mut(), vec_u32_ptr, &mut vec_u32_header)
                        .unwrap();
                    let vec_u32_len = u32::from_le_bytes([
                        vec_u32_header[0],
                        vec_u32_header[1],
                        vec_u32_header[2],
                        vec_u32_header[3],
                    ]) as usize;
                    assert_eq!(vec_u32_len, vec_u32.len(), "vec<u32> length mismatch");

                    // For stack types (u32), the vector stores values directly
                    for (i, &expected_val) in vec_u32.iter().enumerate() {
                        let val_offset = vec_u32_ptr + 8 + (i * 4);
                        let mut val_bytes = [0u8; 4];
                        memory
                            .read(&mut *store.0.borrow_mut(), val_offset, &mut val_bytes)
                            .unwrap();
                        let stored_val = u32::from_le_bytes(val_bytes);
                        assert_eq!(stored_val, expected_val, "vec<u32>[{i}] mismatch");
                    }

                    reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
                },
            );
    }

    #[test]
    fn test_unpack_struct_with_simple_substruct_fuzz() {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(4096));

        let mut compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let struct_type = IntermediateType::IStruct {
            module_id: ModuleId::default(),
            index: 0,
            vm_handled_struct: VmHandledStruct::None,
        };

        let sub_struct_type = IntermediateType::IStruct {
            module_id: ModuleId::default(),
            index: 1,
            vm_handled_struct: VmHandledStruct::None,
        };

        let test_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(0),
            "TestStruct",
            vec![
                (None, IntermediateType::IU16),
                (None, IntermediateType::IU64),
                (None, sub_struct_type.clone()),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let sub_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(1),
            "SubStruct",
            vec![
                (None, IntermediateType::IU32),
                (None, IntermediateType::IU8),
                (None, IntermediateType::IBool),
            ],
            HashMap::new(),
            false,
            IStructType::Common,
        );

        let mut module_data = ModuleData::default();
        module_data.structs.structs = vec![test_struct, sub_struct];
        compilation_ctx.root_module_data = &module_data;

        let mut function_builder =
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();

        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        struct_type
            .add_unpack_instructions(
                Some(&struct_type),
                &mut func_body,
                &mut raw_module,
                None,
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 4096], "test_function", None);

        let memory = instance.get_memory(&mut store, "memory").unwrap();

        let reset_memory = Rc::new(AssertUnwindSafe(
            instance
                .get_typed_func::<(), ()>(&mut store, "reset_memory")
                .unwrap(),
        ));
        let store = Rc::new(AssertUnwindSafe(RefCell::new(store)));
        let entrypoint = Rc::new(AssertUnwindSafe(entrypoint));

        bolero::check!()
            .with_type::<(u16, u64, u32, u8, bool)>()
            .cloned()
            .for_each(|(a, b, sub_a, sub_b, sub_c): (u16, u64, u32, u8, bool)| {
                sol! {
                    struct SubStruct {
                        uint32 a;
                        uint8 b;
                        bool c;
                    }
                    struct TestStruct {
                        uint16 a;
                        uint64 b;
                        SubStruct c;
                    }
                }
                let data = TestStruct {
                    a,
                    b,
                    c: SubStruct {
                        a: sub_a,
                        b: sub_b,
                        c: sub_c,
                    },
                }
                .abi_encode();

                if data.len() > 4096 {
                    return;
                }

                memory
                    .write(
                        &mut *store.0.borrow_mut(),
                        INITIAL_MEMORY_OFFSET as usize,
                        &data,
                    )
                    .unwrap();

                let result_ptr: i32 = entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                // Read the outer struct pointer array (3 pointers)
                let mut struct_data = vec![0; 12];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        result_ptr as usize,
                        &mut struct_data,
                    )
                    .unwrap();

                let u16_ptr = u32::from_le_bytes([
                    struct_data[0],
                    struct_data[1],
                    struct_data[2],
                    struct_data[3],
                ]) as usize;
                let u64_ptr = u32::from_le_bytes([
                    struct_data[4],
                    struct_data[5],
                    struct_data[6],
                    struct_data[7],
                ]) as usize;
                let substruct_ptr = u32::from_le_bytes([
                    struct_data[8],
                    struct_data[9],
                    struct_data[10],
                    struct_data[11],
                ]) as usize;

                let mut u16_data = [0u8; 2];
                memory
                    .read(&mut *store.0.borrow_mut(), u16_ptr, &mut u16_data)
                    .unwrap();
                assert_eq!(u16::from_le_bytes(u16_data), a, "u16 field mismatch");

                let mut u64_data = [0u8; 8];
                memory
                    .read(&mut *store.0.borrow_mut(), u64_ptr, &mut u64_data)
                    .unwrap();
                assert_eq!(u64::from_le_bytes(u64_data), b, "u64 field mismatch");

                // Read the substruct pointer array (3 pointers)
                let mut substruct_data = vec![0; 12];
                memory
                    .read(
                        &mut *store.0.borrow_mut(),
                        substruct_ptr,
                        &mut substruct_data,
                    )
                    .unwrap();

                let sub_u32_ptr = u32::from_le_bytes([
                    substruct_data[0],
                    substruct_data[1],
                    substruct_data[2],
                    substruct_data[3],
                ]) as usize;
                let sub_u8_ptr = u32::from_le_bytes([
                    substruct_data[4],
                    substruct_data[5],
                    substruct_data[6],
                    substruct_data[7],
                ]) as usize;
                let sub_bool_ptr = u32::from_le_bytes([
                    substruct_data[8],
                    substruct_data[9],
                    substruct_data[10],
                    substruct_data[11],
                ]) as usize;

                let mut sub_u32_data = [0u8; 4];
                memory
                    .read(&mut *store.0.borrow_mut(), sub_u32_ptr, &mut sub_u32_data)
                    .unwrap();
                assert_eq!(
                    u32::from_le_bytes(sub_u32_data),
                    sub_a,
                    "substruct u32 field mismatch"
                );

                let mut sub_u8_data = [0u8; 1];
                memory
                    .read(&mut *store.0.borrow_mut(), sub_u8_ptr, &mut sub_u8_data)
                    .unwrap();
                assert_eq!(sub_u8_data[0], sub_b, "substruct u8 field mismatch");

                let mut sub_bool_data = [0u8; 1];
                memory
                    .read(&mut *store.0.borrow_mut(), sub_bool_ptr, &mut sub_bool_data)
                    .unwrap();
                assert_eq!(
                    sub_bool_data[0], sub_c as u8,
                    "substruct bool field mismatch"
                );

                reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
            });
    }

    #[test]
    fn test_unpack_struct_with_dynamic_substruct_fuzz() {
        let (mut raw_module, allocator, memory_id, ctx_globals) = build_module(Some(16384));

        let mut compilation_ctx = test_compilation_context!(memory_id, allocator, ctx_globals);
        let mut runtime_error_data = RuntimeErrorData::new();

        let struct_type = IntermediateType::IStruct {
            module_id: ModuleId::default(),
            index: 0,
            vm_handled_struct: VmHandledStruct::None,
        };

        let sub_struct_type = IntermediateType::IStruct {
            module_id: ModuleId::default(),
            index: 1,
            vm_handled_struct: VmHandledStruct::None,
        };

        let test_struct = IStruct::new(
            move_binary_format::file_format::StructDefinitionIndex(0),
            "TestStruct",
            vec![
                (None, IntermediateType::IU64),
                (None, IntermediateType::IU128),
                (None, sub_struct_type.clone()),
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
            FunctionBuilder::new(&mut raw_module.types, &[], &[ValType::I32]);

        let mut func_body = function_builder.func_body();

        let args_pointer = raw_module.locals.add(ValType::I32);
        let calldata_reader_pointer = raw_module.locals.add(ValType::I32);

        func_body.i32_const(INITIAL_MEMORY_OFFSET);
        func_body.local_tee(args_pointer);
        func_body.local_set(calldata_reader_pointer);

        struct_type
            .add_unpack_instructions(
                Some(&struct_type),
                &mut func_body,
                &mut raw_module,
                None,
                Some(ValType::I32),
                args_pointer,
                calldata_reader_pointer,
                &compilation_ctx,
                Some(&mut runtime_error_data),
                None,
            )
            .unwrap();

        let function = function_builder.finish(vec![], &mut raw_module.funcs);
        raw_module.exports.add("test_function", function);

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut raw_module, vec![0; 8192], "test_function", None);

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
            .with_max_len(10)
            .cloned()
            .for_each(
                |(a, b, vec_u128, vec_u32): (u64, u128, Vec<u128>, Vec<u32>)| {
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
                    let data = TestStruct {
                        a,
                        b,
                        c: SubStruct {
                            x: vec_u128.clone(),
                            y: vec_u32.clone(),
                        },
                    }
                    .abi_encode();

                    if data.len() > 16384 {
                        return;
                    }

                    memory
                        .write(
                            &mut *store.0.borrow_mut(),
                            INITIAL_MEMORY_OFFSET as usize,
                            &data,
                        )
                        .unwrap();

                    // Call the unpack function - may fail with memory access errors for invalid inputs
                    let result_ptr: i32 =
                        entrypoint.0.call(&mut *store.0.borrow_mut(), ()).unwrap();

                    // Read the outer struct pointer array (3 pointers)
                    let mut struct_data = vec![0; 12];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            result_ptr as usize,
                            &mut struct_data,
                        )
                        .unwrap();

                    let u64_ptr = u32::from_le_bytes([
                        struct_data[0],
                        struct_data[1],
                        struct_data[2],
                        struct_data[3],
                    ]) as usize;
                    let u128_ptr = u32::from_le_bytes([
                        struct_data[4],
                        struct_data[5],
                        struct_data[6],
                        struct_data[7],
                    ]) as usize;
                    let substruct_ptr = u32::from_le_bytes([
                        struct_data[8],
                        struct_data[9],
                        struct_data[10],
                        struct_data[11],
                    ]) as usize;

                    let mut u64_data = [0u8; 8];
                    memory
                        .read(&mut *store.0.borrow_mut(), u64_ptr, &mut u64_data)
                        .unwrap();
                    assert_eq!(u64::from_le_bytes(u64_data), a, "u64 field mismatch");

                    let mut u128_data = [0u8; 16];
                    memory
                        .read(&mut *store.0.borrow_mut(), u128_ptr, &mut u128_data)
                        .unwrap();
                    assert_eq!(u128::from_le_bytes(u128_data), b, "u128 field mismatch");

                    // Read the substruct pointer array (2 pointers for 2 vectors)
                    let mut substruct_data = vec![0; 8];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            substruct_ptr,
                            &mut substruct_data,
                        )
                        .unwrap();

                    let vec_u128_ptr = u32::from_le_bytes([
                        substruct_data[0],
                        substruct_data[1],
                        substruct_data[2],
                        substruct_data[3],
                    ]) as usize;
                    let vec_u32_ptr = u32::from_le_bytes([
                        substruct_data[4],
                        substruct_data[5],
                        substruct_data[6],
                        substruct_data[7],
                    ]) as usize;

                    // Verify vec<u128> field
                    let mut vec_u128_header = [0u8; 8];
                    memory
                        .read(
                            &mut *store.0.borrow_mut(),
                            vec_u128_ptr,
                            &mut vec_u128_header,
                        )
                        .unwrap();

                    let vec_u128_len = u32::from_le_bytes([
                        vec_u128_header[0],
                        vec_u128_header[1],
                        vec_u128_header[2],
                        vec_u128_header[3],
                    ]) as usize;
                    assert_eq!(
                        vec_u128_len,
                        vec_u128.len(),
                        "SubStruct vec<u128> length mismatch"
                    );

                    // For vec<u128> (heap type), need to read pointer array first
                    if vec_u128_len > 0 {
                        let ptr_array_ptr = vec_u128_ptr + 8;
                        let mut ptr_array = vec![0u8; vec_u128_len * 4];
                        memory
                            .read(&mut *store.0.borrow_mut(), ptr_array_ptr, &mut ptr_array)
                            .unwrap();

                        for (i, &expected_val) in vec_u128.iter().enumerate() {
                            let val_ptr = u32::from_le_bytes([
                                ptr_array[i * 4],
                                ptr_array[i * 4 + 1],
                                ptr_array[i * 4 + 2],
                                ptr_array[i * 4 + 3],
                            ]) as usize;
                            let mut val_bytes = [0u8; 16];
                            memory
                                .read(&mut *store.0.borrow_mut(), val_ptr, &mut val_bytes)
                                .unwrap();
                            assert_eq!(
                                u128::from_le_bytes(val_bytes),
                                expected_val,
                                "SubStruct vec<u128>[{}] mismatch",
                                i
                            );
                        }
                    }

                    // Verify vec<u32> field
                    let mut vec_u32_header = [0u8; 8];
                    memory
                        .read(&mut *store.0.borrow_mut(), vec_u32_ptr, &mut vec_u32_header)
                        .unwrap();
                    let vec_u32_len = u32::from_le_bytes([
                        vec_u32_header[0],
                        vec_u32_header[1],
                        vec_u32_header[2],
                        vec_u32_header[3],
                    ]) as usize;
                    assert_eq!(
                        vec_u32_len,
                        vec_u32.len(),
                        "SubStruct vec<u32> length mismatch"
                    );

                    // For vec<u32> (stack type), values are stored directly after header
                    if vec_u32_len > 0 {
                        let vec_u32_data_ptr = vec_u32_ptr + 8;
                        for (i, &expected_val) in vec_u32.iter().enumerate() {
                            let val_offset = vec_u32_data_ptr + (i * 4);
                            let mut val_bytes = [0u8; 4];
                            memory
                                .read(&mut *store.0.borrow_mut(), val_offset, &mut val_bytes)
                                .unwrap();

                            assert_eq!(
                                u32::from_le_bytes(val_bytes),
                                expected_val,
                                "SubStruct vec<u32>[{}] mismatch",
                                i
                            );
                        }
                    }

                    reset_memory.0.call(&mut *store.0.borrow_mut(), ()).unwrap();
                },
            );
    }
}
