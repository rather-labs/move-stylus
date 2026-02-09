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
/// Parameters:
///   - `uid_ptr` (i32): pointer to the object's UID
///   - `unpack_frozen` (i32): whether to also search frozen objects (1 = yes, 0 = no).
///     Only used when `object_kind` is not specified.
///   - `object_kind` (i32): which storage mapping to use directly:
///
/// | Value | Meaning        | Lookup function used            |
/// |-------|----------------|---------------------------------|
/// |   0   | Owned          | `LocateStorageOwnedData(uid)`   |
/// |   1   | Shared         | `LocateStorageSharedData(uid)`  |
/// |   2   | Frozen         | `LocateStorageFrozenData(uid)`  |
/// |  -1   | Not specified  | `LocateStorageData(uid, unpack_frozen)` |
///
/// When the kind is explicitly known (via `#[owned_objects]`, `#[shared_objects]`, or
/// `#[frozen_objects]` modifiers), the generated code saves gas by going directly to the
/// correct storage mapping instead of searching multiple mappings sequentially.
pub fn unpack_storage_struct_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::UnpackStorageStruct
        .get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name).func_body();

    // Arguments
    let uid_ptr = module.locals.add(ValType::I32);
    let unpack_frozen = module.locals.add(ValType::I32);
    let object_kind = module.locals.add(ValType::I32);
    let owner_id_ptr = module.locals.add(ValType::I32);

    // Resolve all locate_storage runtime functions upfront
    let locate_storage_data_fn = RuntimeFunction::LocateStorageData.get(
        module,
        Some(compilation_ctx),
        Some(runtime_error_data),
    )?;
    let locate_owned_fn = RuntimeFunction::LocateStorageOwnedData.get(
        module,
        Some(compilation_ctx),
        Some(runtime_error_data),
    )?;
    let locate_shared_fn = RuntimeFunction::LocateStorageSharedData.get(
        module,
        Some(compilation_ctx),
        Some(runtime_error_data),
    )?;
    let locate_frozen_fn = RuntimeFunction::LocateStorageFrozenData.get(
        module,
        Some(compilation_ctx),
        Some(runtime_error_data),
    )?;

    // Dispatch to the appropriate locate_storage function based on object_kind.
    //
    // if object_kind == OWNED → LocateStorageOwnedData(uid_ptr)
    // else if object_kind == SHARED → LocateStorageSharedData(uid_ptr)
    // else if object_kind == FROZEN → LocateStorageFrozenData(uid_ptr)
    // else → LocateStorageData(uid_ptr, unpack_frozen)
    builder
        .local_get(object_kind)
        .i32_const(ObjectKind::Owned as i32)
        .binop(BinaryOp::I32Eq)
        .if_else(
            ValType::I32,
            |then_| {
                then_.local_get(uid_ptr).call_runtime_function(
                    compilation_ctx,
                    locate_owned_fn,
                    &RuntimeFunction::LocateStorageOwnedData,
                    Some(ValType::I32),
                );
            },
            |else_| {
                else_
                    .local_get(object_kind)
                    .i32_const(ObjectKind::Shared as i32)
                    .binop(BinaryOp::I32Eq)
                    .if_else(
                        ValType::I32,
                        |then_| {
                            then_.local_get(uid_ptr).call_runtime_function(
                                compilation_ctx,
                                locate_shared_fn,
                                &RuntimeFunction::LocateStorageSharedData,
                                Some(ValType::I32),
                            );
                        },
                        |else_| {
                            else_
                                .local_get(object_kind)
                                .i32_const(ObjectKind::Frozen as i32)
                                .binop(BinaryOp::I32Eq)
                                .if_else(
                                    ValType::I32,
                                    |then_| {
                                        then_.local_get(uid_ptr).call_runtime_function(
                                            compilation_ctx,
                                            locate_frozen_fn,
                                            &RuntimeFunction::LocateStorageFrozenData,
                                            Some(ValType::I32),
                                        );
                                    },
                                    |else_| {
                                        // Default: LocateStorageData(uid_ptr, unpack_frozen)
                                        else_
                                            .local_get(uid_ptr)
                                            .local_get(unpack_frozen)
                                            .call_runtime_function(
                                                compilation_ctx,
                                                locate_storage_data_fn,
                                                &RuntimeFunction::LocateStorageData,
                                                Some(ValType::I32),
                                            );
                                    },
                                );
                        },
                    );
            },
        );

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

    Ok(function.finish(vec![uid_ptr, unpack_frozen, object_kind], &mut module.funcs))
}
