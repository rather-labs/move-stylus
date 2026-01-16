use super::{RuntimeFunction, error::RuntimeFunctionError};
use crate::{
    CompilationContext,
    data::{
        DATA_DERIVED_MAPPING_SLOT, DATA_FROZEN_OBJECTS_KEY_OFFSET,
        DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET, DATA_OBJECTS_SLOT_OFFSET,
        DATA_SHARED_OBJECTS_KEY_OFFSET, DATA_SLOT_DATA_PTR_OFFSET,
        DATA_STORAGE_OBJECT_OWNER_OFFSET, DATA_U256_ONE_OFFSET, DATA_ZERO_OFFSET, RuntimeErrorData,
    },
    error::RuntimeError,
    hostio::host_functions::{
        self, storage_cache_bytes32, storage_flush_cache, storage_load_bytes32, tx_origin,
    },
    storage::{
        common::add_delete_storage_struct_instructions,
        decoding::add_read_and_decode_storage_struct_instructions,
        encoding::add_encode_and_save_into_storage_struct_instructions,
    },
    translation::intermediate_types::{IntermediateType, heap_integers::IU256},
    wasm_builder_extensions::WasmBuilderExtension,
};
use walrus::{
    FunctionBuilder, FunctionId, GlobalId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind, UnaryOp},
};

/// Looks for an struct inside the objects mappings. The objects mappings follows the solidity notation:
/// mapping(bytes32 => mapping(bytes32 => T)) public moveObjects;
///
/// Where:
/// * The outer mapping key is the id of the owner (could be an address or object id).
/// * The inner mapping key is the object id itself.
/// * The value is the encoded structure.
///
/// The lookup is done in the following order:
/// * In the shared objects key (1)
/// * In the signer's owned objects (key is the signer's address).
/// * In the frozen objects key (2)
///
/// If no data is found an unrechable error is thrown. Otherwise the slot number to reconstruct the
/// struct is written in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET.
///
/// # WASM Function Arguments
/// * `uid_ptr` - (i32): pointer to the 32 bytes object id
/// * `search_frozen` - (i32): if non-zero, search in frozen objects as well
///
/// # WASM Function Returns
/// * (i32): pointer to the owner id
pub fn locate_storage_data(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Runtime functions
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx), None)?;
    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx), None)?;

    // Host functions
    let (tx_origin, _) = tx_origin(module);
    let (storage_load, _) = storage_load_bytes32(module);

    // Function declaration
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function
        .name(RuntimeFunction::LocateStorageData.name().to_owned())
        .func_body();

    // Arguments
    let uid_ptr = module.locals.add(ValType::I32);
    let search_frozen = module.locals.add(ValType::I32);

    builder.block(ValType::I32, |block| {
        let exit_block = block.id();

        // ==
        // Shared objects
        // ==

        block
            .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
            .local_get(uid_ptr)
            .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
            .call(write_object_slot_fn);

        // Load data from slot
        block
            .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_load);

        // Check if it is empty (all zeroes)
        block
            .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .i32_const(32)
            .call(is_zero_fn)
            .negate()
            .br_if(exit_block)
            .drop();

        // ==
        // Signer's objects
        // ==

        // Wipe the first 12 bytes, and then write the tx signer address
        block
            .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
            .i32_const(0)
            .i32_const(12)
            .memory_fill(compilation_ctx.memory_id);

        // Write the tx signer (20 bytes) left padded
        block
            .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET + 12)
            .call(tx_origin);

        block
            .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
            .local_get(uid_ptr)
            .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
            .call(write_object_slot_fn);

        // Load data from slot
        block
            .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_load);

        // Check if it is empty (all zeroes)
        block
            .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .i32_const(32)
            .call(is_zero_fn)
            .negate()
            .br_if(exit_block)
            .drop();

        // ==
        // Frozen objects
        // ==
        // Copy the frozen objects key to the owners offset
        block.block(None, |frozen_block| {
            let exit_frozen_block = frozen_block.id();
            frozen_block
                .local_get(search_frozen)
                .unop(UnaryOp::I32Eqz)
                .br_if(exit_frozen_block);

            frozen_block
                .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
                .local_get(uid_ptr)
                .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                .call(write_object_slot_fn);

            // Load data from slot
            frozen_block
                .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(storage_load);

            // Check if it is empty (all zeroes)
            frozen_block
                .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(32)
                .call(is_zero_fn)
                .negate()
                .br_if(exit_block)
                .drop();
        });

        // If we get here means the object was not found
        block.return_error(
            module,
            compilation_ctx,
            runtime_error_data,
            RuntimeError::StorageObjectNotFound,
        );
    });

    Ok(function.finish(vec![uid_ptr, search_frozen], &mut module.funcs))
}

/// Looks for an struct inside the object's owner namespace. The objects mappings follows the
/// solidity notation:
///
/// mapping(bytes32 => mapping(bytes32 => T)) public moveObjects;
///
/// Where:
/// * The outer mapping key is the id of the owner.
/// * The inner mapping key is the object id itself.
/// * The value is the encoded structure.
///
/// If no data is found an unrechable error is thrown. Otherwise the slot number to reconstruct the
/// struct is written in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET.
///
/// # WASM Function Arguments
/// * `uid_ptr` - (i32): pointer to the 32 bytes object id
///
/// # WASM Function Returns
/// * (i32): pointer to the owner id
pub fn locate_storage_owned_data(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Runtime functions
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx), None)?;
    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx), None)?;

    // Host functions
    let (tx_origin, _) = tx_origin(module);
    let (storage_load, _) = storage_load_bytes32(module);

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::LocateStorageOwnedData.name().to_owned())
        .func_body();

    // Arguments
    let uid_ptr = module.locals.add(ValType::I32);

    // Wipe the first 12 bytes, and then write the tx signer address
    builder
        .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
        .i32_const(0)
        .i32_const(12)
        .memory_fill(compilation_ctx.memory_id);

    // Write the tx signer (20 bytes) left padded
    builder
        .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET + 12)
        .call(tx_origin);

    builder
        .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
        .local_get(uid_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .call(write_object_slot_fn);

    // Load data from slot
    builder
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_load);

    // Check if it is empty (all zeroes)
    builder
        .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(32)
        .call(is_zero_fn)
        .negate()
        .return_()
        .drop();

    // If we get here means the object was not found
    builder.return_error(
        module,
        compilation_ctx,
        runtime_error_data,
        RuntimeError::StorageObjectNotFound,
    );

    Ok(function.finish(vec![uid_ptr], &mut module.funcs))
}

/// Looks for an struct inside the object's shared namespace. The objects mappings follows the
/// solidity notation:
///
/// mapping(bytes32 => mapping(bytes32 => T)) public moveObjects;
///
/// Where:
/// * The outer mapping key is the id of the owner (shared key, address 0x1).
/// * The inner mapping key is the object id itself.
/// * The value is the encoded structure.
///
/// If no data is found an unrechable error is thrown. Otherwise the slot number to reconstruct the
/// struct is written in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET.
///
/// # WASM Function Arguments
/// * `uid_ptr` - (i32): pointer to the 32 bytes object id
///
/// # WASM Function Returns
/// * (i32): pointer to the owner id
pub fn locate_storage_shared_data(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Runtime functions
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx), None)?;
    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx), None)?;

    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::LocateStorageSharedData.name().to_owned())
        .func_body();

    // Arguments
    let uid_ptr = module.locals.add(ValType::I32);

    builder
        .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
        .local_get(uid_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .call(write_object_slot_fn);

    // Load data from slot
    builder
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_load);

    // Check if it is empty (all zeroes)
    builder
        .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(32)
        .call(is_zero_fn)
        .negate()
        .return_()
        .drop();

    // If we get here means the object was not found
    builder.return_error(
        module,
        compilation_ctx,
        runtime_error_data,
        RuntimeError::StorageObjectNotFound,
    );

    Ok(function.finish(vec![uid_ptr], &mut module.funcs))
}

/// Looks for an struct inside the object's frozen namespace. The objects mappings follows the
/// solidity notation:
///
/// mapping(bytes32 => mapping(bytes32 => T)) public moveObjects;
///
/// Where:
/// * The outer mapping key is the id of the owner (frozen key, address 0x2).
/// * The inner mapping key is the object id itself.
/// * The value is the encoded structure.
///
/// If no data is found an unrechable error is thrown. Otherwise the slot number to reconstruct the
/// struct is written in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET.
///
/// # WASM Function Arguments
/// * `uid_ptr` - (i32): pointer to the 32 bytes object id
///
/// # WASM Function Returns
/// * (i32): pointer to the owner id
pub fn locate_storage_frozen_data(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
) -> Result<FunctionId, RuntimeFunctionError> {
    // Runtime functions
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx), None)?;
    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx), None)?;

    // Host functions
    let (storage_load, _) = storage_load_bytes32(module);

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::LocateStorageFrozenData.name().to_owned())
        .func_body();

    // Arguments
    let uid_ptr = module.locals.add(ValType::I32);

    builder
        .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
        .local_get(uid_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .call(write_object_slot_fn);

    // Load data from slot
    builder
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .call(storage_load);

    // Check if it is empty (all zeroes)
    builder
        .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
        .i32_const(32)
        .call(is_zero_fn)
        .negate()
        .return_()
        .drop();

    // If we get here means the object was not found
    builder.return_error(
        module,
        compilation_ctx,
        runtime_error_data,
        RuntimeError::StorageObjectNotFound,
    );

    Ok(function.finish(vec![uid_ptr], &mut module.funcs))
}

/// Computes the storage slot number where the struct should be persisted.
///
/// When working with a struct in memory that has the `key` ability,
/// once processing is complete, its storage slot must be calculated
/// so the changes can be saved.
///
/// The slot number is written in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET.
///
/// # WASM Function Arguments
/// * `struct_ptr` - (i32): pointer to the struct
pub fn locate_struct_slot(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);

    let mut builder = function
        .name(RuntimeFunction::LocateStructSlot.name().to_owned())
        .func_body();

    let write_object_slot_fn =
        RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx), None)?;
    let get_id_bytes_ptr_fn =
        RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx), None)?;
    let get_struct_owner_fn = RuntimeFunction::GetStructOwner.get(module, None, None)?;

    let struct_ptr = module.locals.add(ValType::I32);

    // Obtain this object's owner
    builder.local_get(struct_ptr).call(get_struct_owner_fn);

    // Get the pointer to the 32 bytes holding the data of the id
    builder.local_get(struct_ptr).call(get_id_bytes_ptr_fn);

    builder.i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET);

    // Compute the slot where it should be saved
    builder.call(write_object_slot_fn);

    Ok(function.finish(vec![struct_ptr], &mut module.funcs))
}

/// Calculates the slot from the slot mapping
///
/// # WASM Function Arguments
/// * `owner_ptr` - (i32): pointer to the 32 bytes owner id
/// * `uid_ptr` - (i32): pointer to the 32 bytes object id
/// * `slot_ptr` - (i32): pointer where the slot number is going to be written
pub fn write_object_slot(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );
    let mut builder = function
        .name(RuntimeFunction::WriteObjectSlot.name().to_owned())
        .func_body();

    let uid_ptr = module.locals.add(ValType::I32);
    let owner_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    // Calculate the slot address
    let derive_slot_fn =
        RuntimeFunction::DeriveMappingSlot.get(module, Some(compilation_ctx), None)?;

    // Derive the slot for the first mapping
    builder
        .i32_const(DATA_OBJECTS_SLOT_OFFSET)
        .local_get(owner_ptr)
        .local_get(slot_ptr)
        .call(derive_slot_fn);

    // Derive slot for the second mapping
    builder
        .local_get(slot_ptr)
        .local_get(uid_ptr)
        .local_get(slot_ptr)
        .call(derive_slot_fn);

    Ok(function.finish(vec![owner_ptr, uid_ptr, slot_ptr], &mut module.funcs))
}

pub fn storage_next_slot_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::StorageNextSlot.name().to_owned())
        .func_body();

    let slot_ptr = module.locals.add(ValType::I32);

    let swap_256_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx), None)?;
    let add_u256_fn = RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx), None)?;

    // BE to LE ptr so we can make the addition
    builder
        .local_get(slot_ptr)
        .local_get(slot_ptr)
        .call(swap_256_fn);

    // Add one to slot
    builder
        .local_get(slot_ptr)
        .i32_const(DATA_U256_ONE_OFFSET)
        .local_get(slot_ptr)
        .i32_const(32)
        .call(add_u256_fn);

    // LE to BE ptr so we can use the storage function
    builder
        .local_get(slot_ptr)
        .local_get(slot_ptr)
        .call(swap_256_fn);

    Ok(function.finish(vec![slot_ptr], &mut module.funcs))
}

/// This function returns a pointer to the 32 bytes holding the data of the id, given a struct pointer as input
pub fn get_id_bytes_ptr(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::GetIdBytesPtr.name().to_owned())
        .func_body();

    let struct_ptr = module.locals.add(ValType::I32);

    // Obtain the object's id, it must be the first field containing a UID struct
    // The UID struct has the following form
    //
    // UID { id: ID { bytes: <bytes> } }
    //
    // The first load instruction puts in stack the first pointer value of the strucure, that is a
    // pointer to the UID struct
    //
    // The second load instruction puts in stack the pointer to the ID struct
    //
    // The third load instruction loads the ID's bytes field pointer
    //
    // At the end of the load chain we point to the 32 bytes holding the data
    builder
        .local_get(struct_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        );

    function.finish(vec![struct_ptr], &mut module.funcs)
}

/// The value corresponding to a mapping key k is located at keccak256(h(k) . p) where . is concatenation
/// and h is a function that is applied to the key depending on its type:
/// * for value types, h pads the value to 32 bytes in the same way as when storing the value in memory.
/// * for strings and byte arrays, h(k) is just the unpadded data.
///
/// # WASM Function Arguments
/// * `mapping_slot_ptr` - (i32): pointer to the mapping slot (32 bytes)
/// * `key_ptr` - (i32): pointer to the key (32 bytes)
/// * `derived_slot_ptr` - (i32): pointer to the derived slot (32 bytes)
pub fn derive_mapping_slot(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[],
    );

    let mut builder = function
        .name(RuntimeFunction::DeriveMappingSlot.name().to_owned())
        .func_body();

    // Arguments locals
    let mapping_slot_ptr = module.locals.add(ValType::I32);
    let key_ptr = module.locals.add(ValType::I32);
    let derived_slot_ptr = module.locals.add(ValType::I32);

    let (native_keccak, _) = host_functions::native_keccak256(module);

    builder
        .i32_const(DATA_DERIVED_MAPPING_SLOT)
        .local_get(key_ptr)
        .i32_const(32) // copy 32 bytes, for now fixed size
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    builder
        .i32_const(DATA_DERIVED_MAPPING_SLOT + 32)
        .local_get(mapping_slot_ptr)
        .i32_const(32) // copy 32 bytes
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Hash the data, this is the mapping slot we are looking for -> v = keccak256(h(k) . p)
    builder
        .i32_const(DATA_DERIVED_MAPPING_SLOT)
        .i32_const(64)
        .local_get(derived_slot_ptr)
        .call(native_keccak);

    function.finish(
        vec![mapping_slot_ptr, key_ptr, derived_slot_ptr],
        &mut module.funcs,
    )
}

/// Calculates the storage slot for an element in a dynamic array at a specified index,
/// using Solidity's storage layout convention:
///   base = keccak256(p)
///   element_slot = base + index * element_size_in_slots
///
/// # WASM Function Arguments
/// * `array_slot_ptr` - (i32): pointer to the u256 slot `p`, which is the header slot of the array.
/// * `elem_index` - (i32): u32 value representing the element's index in the array (little-endian).
/// * `elem_size` - (i32): u32 value representing the size of each element in bytes (little-endian).
/// * `derived_elem_slot_ptr` - (i32): pointer to the resulting u256 slot where the element is stored.
///
/// NOTE: The computed u256 slot value for the element, in big-endian format, is stored at
/// `derived_elem_slot_ptr`.
pub fn derive_dyn_array_slot(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
        &[],
    );

    let mut builder = function
        .name(RuntimeFunction::DeriveDynArraySlot.name().to_owned())
        .func_body();

    // Arguments locals
    let array_slot_ptr = module.locals.add(ValType::I32);
    let elem_index = module.locals.add(ValType::I32);
    let elem_size = module.locals.add(ValType::I32);
    let derived_elem_slot_ptr = module.locals.add(ValType::I32);

    let (native_keccak, _) = host_functions::native_keccak256(module);
    let swap_i32_bytes_fn = RuntimeFunction::SwapI32Bytes.get(module, None, None)?;
    let add_u256_fn = RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx), None)?;

    // Guard: check elem_size is greater than 0
    builder
        .local_get(elem_size)
        .i32_const(0)
        .binop(BinaryOp::I32LeU)
        .if_else(
            None,
            |then| {
                then.unreachable();
            },
            |_else| {},
        );

    // Local for the pointer to keccak256(p)
    let base_slot_ptr = module.locals.add(ValType::I32);

    // Allocate memory for the base slot result
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(base_slot_ptr);

    // Compute base = keccak256(p)
    builder
        .local_get(array_slot_ptr)
        .i32_const(32)
        .local_get(base_slot_ptr)
        .call(native_keccak);

    // Check if the element size is less than 32 bytes, i.e. it fits in a storage slot
    builder
        .local_get(elem_size)
        .i32_const(32)
        .binop(BinaryOp::I32LtU);

    builder.if_else(
        ValType::I32,
        |then| {
            // Case: Element fits within a single 32-byte (256-bit) storage slot
            //
            // Solidity packs multiple elements per slot when element size < 32 bytes.
            // We need to compute the slot offset where the element is stored:
            //
            // offset = floor(index / floor(32 / elem_size))
            //
            // Step 1: Get the index (u32)
            then.local_get(elem_index);

            // Step 2: Get the element size and compute divisor = floor(32 / elem_size)
            then.i32_const(32)
                .local_get(elem_size)
                .binop(BinaryOp::I32DivU);

            // Step 3: Compute offset = floor(index / divisor)
            then.binop(BinaryOp::I32DivU);
        },
        |else_| {
            // Case: Element does NOT fit within a single storage slot (elem_size ≥ 32 bytes)
            //
            // Solidity stores each element in full slots and does NOT pack them.
            // We compute how many slots each element needs:
            //
            // slots_per_element = ceil(elem_size / 32) = (elem_size + 31) / 32
            // offset = index * slots_per_element
            //
            // Step 1: Get the index (u32)
            else_.local_get(elem_index);

            // Step 2: Compute slots_per_element = (elem_size + 31) / 32
            else_
                .local_get(elem_size)
                .i32_const(31)
                .binop(BinaryOp::I32Add)
                .i32_const(32)
                .binop(BinaryOp::I32DivU);

            // Step 3: Multiply to get offset = index * slots_per_element
            else_.binop(BinaryOp::I32Mul);
        },
    );

    // Convert to big-endian
    builder.call(swap_i32_bytes_fn);

    // Repurpose elem_size_ptr to hold the result (i.e., offset as I32)
    let elem_offset_32 = elem_size;
    builder.local_set(elem_offset_32);

    // Repurpose elem_index to allocate and hold the offset as U256
    let elem_offset_256 = elem_index;
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(elem_offset_256)
        .local_get(elem_offset_256)
        .local_get(elem_offset_32)
        // Store the u32 big-endian offset at the last 4 bytes of the memory to convert it to u256
        .store(
            compilation_ctx.memory_id,
            StoreKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 28,
            },
        );

    // Add base + offset → final element slot
    builder
        .local_get(derived_elem_slot_ptr)
        .local_get(elem_offset_256)
        .local_get(base_slot_ptr)
        .i32_const(IU256::HEAP_SIZE)
        .call(compilation_ctx.allocator)
        .i32_const(IU256::HEAP_SIZE)
        .call(add_u256_fn);

    builder // copy add(base, offset) result to #derived_elem_slot_ptr
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    Ok(function.finish(
        vec![array_slot_ptr, elem_index, elem_size, derived_elem_slot_ptr],
        &mut module.funcs,
    ))
}

/// Generates a function that encodes and saves an specific struct into the storage.
///
/// WASM Function Arguments:
/// * `struct_ptr` - (i32): pointer to the struct
/// * `slot_ptr` - (i32): pointer to the storage slot
pub fn add_encode_and_save_into_storage_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::EncodeAndSaveInStorage
        .get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    // Arguments
    let struct_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    // Locals
    let slot_offset = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(slot_offset);

    add_encode_and_save_into_storage_struct_instructions(
        module,
        &mut builder,
        compilation_ctx,
        runtime_error_data,
        struct_ptr,
        slot_ptr,
        slot_offset,
        None,
        itype,
    )?;

    Ok(function.finish(vec![struct_ptr, slot_ptr], &mut module.funcs))
}

/// Generates a function that reads an specific struct from the storage.
///
/// This function:
/// 1. Locates the storage slot of the object.
/// 2. Reads and decodes the struct from storage.
/// 3. Returns a pointer to the in-memory representation of the struct.
///
/// # WASM Function Arguments
/// * `slot_ptr` - (i32): pointer to the storage slot
/// * `uid_ptr` - (i32): pointer to the UID
/// * `owner_ptr` - (i32): pointer to the owner id
///
/// # WASM Function Returns
/// * `struct_ptr` - (i32): pointer to the struct
pub fn add_read_and_decode_from_storage_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::ReadAndDecodeFromStorage
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
    let slot_ptr = module.locals.add(ValType::I32);
    let struct_id_ptr = module.locals.add(ValType::I32);
    let owner_ptr = module.locals.add(ValType::I32);

    // Locals

    let slot_offset = module.locals.add(ValType::I32);
    builder.i32_const(0).local_set(slot_offset);

    let struct_ptr = add_read_and_decode_storage_struct_instructions(
        module,
        &mut builder,
        compilation_ctx,
        runtime_error_data,
        slot_ptr,
        slot_offset,
        owner_ptr,
        Some(struct_id_ptr),
        itype,
    )?;

    builder.local_get(struct_ptr);

    Ok(function.finish(vec![slot_ptr, struct_id_ptr, owner_ptr], &mut module.funcs))
}

/// Generates a function that deletes an object from storage.
///
/// This function:
/// 1. Validates the object is not frozen (frozen objects cannot be deleted).
/// 2. Locates the storage slot of the object.
/// 3. Clears the storage slot and any additional slots occupied by the struct fields.
/// 4. Flushes the cache to finalize the deletion.
///
/// # WASM Function Arguments
/// * `struct_ptr` - (i32): pointer to the struct
pub fn add_delete_struct_from_storage_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name =
        RuntimeFunction::DeleteFromStorage.get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let struct_ = compilation_ctx.get_struct_by_intermediate_type(itype)?;

    let locate_struct_slot_fn =
        RuntimeFunction::LocateStructSlot.get(module, Some(compilation_ctx), None)?;
    let get_struct_owner_fn = RuntimeFunction::GetStructOwner.get(module, None, None)?;
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx), None)?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let struct_ptr = module.locals.add(ValType::I32);

    // Verify if the object is frozen; if not, continue.
    builder
        .local_get(struct_ptr)
        .call(get_struct_owner_fn)
        .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
        .i32_const(32)
        .call(equality_fn);

    let mut inner_result = Ok(());
    builder.if_else(
        None,
        |then| {
            // Emit an unreachable if the object is frozen
            then.unreachable();
        },
        |else_| {
            // Wipe the slot data placeholder. We will use it to erase the slots in the storage
            else_
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(0)
                .i32_const(32)
                .memory_fill(compilation_ctx.memory_id);

            // Locals
            let slot_ptr = module.locals.add(ValType::I32);

            // Calculate the object slot in storage
            else_
                .local_get(struct_ptr)
                .call(locate_struct_slot_fn)
                .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                .local_set(slot_ptr);

            // Initialize the number of bytes used in the slot to 8, to reflect the 8-byte type hash
            let slot_offset = module.locals.add(ValType::I32);
            else_.i32_const(8).local_set(slot_offset);

            // Delete the struct from storage
            inner_result = add_delete_storage_struct_instructions(
                module,
                else_,
                compilation_ctx,
                runtime_error_data,
                slot_ptr,
                slot_offset,
                &struct_,
            );
        },
    );
    inner_result?;

    // Wipe out the owner
    builder
        .local_get(struct_ptr)
        .call(get_struct_owner_fn)
        .i32_const(0)
        .i32_const(32)
        .memory_fill(compilation_ctx.memory_id);

    Ok(function.finish(vec![struct_ptr], &mut module.funcs))
}

/// Remove any objects that have been recently transferred into the struct (transfer-to-object feature or TTO)
/// from the original owner's mapping in storage.
///
/// Example: The Object 'obj' is initially owned by the sender. It is then passed as a value to the 'request_swap' function,
/// where it gets wrapped within the 'SwapRequest' struct. This struct is subsequently transferred to the service address.
/// In this scenario, 'obj' must be removed from the sender's ownership mapping (the original owner) because the 'SwapRequest' struct is now the actual owner.
///```ignore
/// public fun request_swap(
///     obj: Object,
///   service: address,
///     fee: u64,
///     ctx: &mut TxContext,
/// ) {
///     assert!(fee >= MIN_FEE, EFeeTooLow);
///
///    let request = SwapRequest {
///         id: object::new(ctx),
///        owner: ctx.sender(),
///         object: obj,
///         fee,
///     };
///
///    transfer::transfer(request, service)
/// }
///```
/// # WASM Function Arguments
/// * `parent_struct_ptr` - (i32): pointer to the parent struct
pub fn add_check_and_delete_struct_tto_fields_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::CheckAndDeleteStructTtoFields
        .get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    // Arguments
    let parent_struct_ptr = module.locals.add(ValType::I32);

    let struct_ = compilation_ctx.get_struct_by_intermediate_type(itype)?;

    // Iterate over the fields of the struct
    let mut offset: i32 = 0;
    for field in struct_.fields.iter() {
        if matches!(
            field,
            IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. }
        ) {
            let child_struct_ptr = module.locals.add(ValType::I32);

            // Get the pointer to the child struct
            builder
                .local_get(parent_struct_ptr)
                // Load the intermediate pointer to the child struct
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: offset as u32,
                    },
                )
                .local_set(child_struct_ptr);

            // Call the function recursively to delete any recently tto objects within the child struct
            let delete_tto_objects_fn = RuntimeFunction::CheckAndDeleteStructTtoFields
                .get_generic(module, compilation_ctx, Some(runtime_error_data), &[field])?;
            builder
                .local_get(child_struct_ptr)
                .call(delete_tto_objects_fn);

            // If the child struct has key, remove it from the original owner's storage if it's still there.
            let delete_tto_object_fn = RuntimeFunction::DeleteTtoObject.get_generic(
                module,
                compilation_ctx,
                Some(runtime_error_data),
                &[field],
            )?;
            builder
                .local_get(parent_struct_ptr)
                .local_get(child_struct_ptr)
                .call(delete_tto_object_fn);
        } else if let IntermediateType::IVector(inner) = field {
            if matches!(
                inner.as_ref(),
                IntermediateType::IStruct { .. } | IntermediateType::IGenericStructInstance { .. }
            ) {
                let vector_ptr = module.locals.add(ValType::I32);
                let len = module.locals.add(ValType::I32);

                // Get the pointer to the vector
                builder
                    .local_get(parent_struct_ptr)
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: offset as u32,
                        },
                    )
                    .local_tee(vector_ptr);

                // Load vector length from its header
                builder
                    .load(
                        compilation_ctx.memory_id,
                        LoadKind::I32 { atomic: false },
                        MemArg {
                            align: 0,
                            offset: 0,
                        },
                    )
                    .local_set(len);

                let delete_tto_objects_fn = RuntimeFunction::CheckAndDeleteStructTtoFields
                    .get_generic(
                        module,
                        compilation_ctx,
                        Some(runtime_error_data),
                        &[inner.as_ref()],
                    )?;

                let delete_tto_object_fn = RuntimeFunction::DeleteTtoObject.get_generic(
                    module,
                    compilation_ctx,
                    Some(runtime_error_data),
                    &[inner.as_ref()],
                )?;
                // Outer block: if the vector length is 0, we skip to the end
                builder.block(None, |outer_block| {
                    let outer_block_id = outer_block.id();

                    // Check if length == 0
                    outer_block
                        .local_get(len)
                        .unop(UnaryOp::I32Eqz)
                        .br_if(outer_block_id);

                    outer_block.block(None, |inner_block| {
                        let inner_block_id = inner_block.id();

                        let i = module.locals.add(ValType::I32);
                        let elem_ptr = module.locals.add(ValType::I32);

                        // Set the aux locals to 0 to start the loop
                        inner_block.i32_const(0).local_set(i);
                        inner_block.loop_(None, |loop_| {
                            let loop_id = loop_.id();

                            loop_
                                .vec_elem_ptr(vector_ptr, i, 4)
                                .load(
                                    compilation_ctx.memory_id,
                                    LoadKind::I32 { atomic: false },
                                    MemArg {
                                        align: 0,
                                        offset: 0,
                                    },
                                )
                                .local_set(elem_ptr);

                            // Call the function recursively to delete any recently tto objects within the vector element struct

                            loop_.local_get(elem_ptr).call(delete_tto_objects_fn);

                            loop_
                                .local_get(parent_struct_ptr)
                                .local_get(elem_ptr)
                                .call(delete_tto_object_fn);

                            // Exit after processing all elements
                            loop_
                                .local_get(i)
                                .local_get(len)
                                .i32_const(1)
                                .binop(BinaryOp::I32Sub)
                                .binop(BinaryOp::I32Eq)
                                .br_if(inner_block_id);

                            // i = i + 1 and continue the loop
                            loop_
                                .local_get(i)
                                .i32_const(1)
                                .binop(BinaryOp::I32Add)
                                .local_set(i)
                                .br(loop_id);
                        });
                    });
                });
            }
        }
        offset += 4;
    }

    Ok(function.finish(vec![parent_struct_ptr], &mut module.funcs))
}

/// This function deletes a recently transferred wrapped object from the original owner's storage.
///
/// # WASM Function Arguments
/// * `parent_struct_ptr` - (i32): pointer to the parent struct
/// * `child_struct_ptr` - (i32): pointer to the child struct
pub fn add_delete_tto_object_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name =
        RuntimeFunction::DeleteTtoObject.get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    };

    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx), None)?;
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx), None)?;
    let get_id_bytes_ptr_fn =
        RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx), None)?;
    let get_struct_owner_fn = RuntimeFunction::GetStructOwner.get(module, None, None)?;
    let delete_wrapped_object_fn = RuntimeFunction::DeleteFromStorage.get_generic(
        module,
        compilation_ctx,
        Some(runtime_error_data),
        &[itype],
    )?;

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let parent_struct_ptr = module.locals.add(ValType::I32);
    let child_struct_ptr = module.locals.add(ValType::I32);

    let struct_ = compilation_ctx.get_struct_by_intermediate_type(itype)?;

    // If the child struct has key, remove it from the original owner's storage if it's still there.
    if struct_.has_key {
        builder.block(None, |block| {
            let block_id = block.id();

            let child_struct_owner_ptr = module.locals.add(ValType::I32);

            // Get the pointer to the child struct owner
            block
                .local_get(child_struct_ptr)
                .call(get_struct_owner_fn)
                .local_set(child_struct_owner_ptr);

            // Check if the owner is zero (means there's no owner, so we don't need to delete anything)
            // This can happen if we have just created the struct and not transfered it yet.
            block
                .local_get(child_struct_owner_ptr)
                .i32_const(32)
                .call(is_zero_fn)
                .br_if(block_id);

            // Verify if the owner of the child struct matches the ID of the parent struct.
            // If they differ, it indicates that the child struct has been just wrapped into the parent struct
            // and should be removed from the original owner's storage mapping.
            block
                .local_get(parent_struct_ptr)
                .call(get_id_bytes_ptr_fn)
                .local_get(child_struct_owner_ptr)
                .i32_const(32)
                .call(equality_fn)
                .br_if(block_id);

            // Get the delete function for the child struct
            block
                .local_get(child_struct_ptr)
                .call(delete_wrapped_object_fn);
        });
    }

    Ok(function.finish(vec![parent_struct_ptr, child_struct_ptr], &mut module.funcs))
}

/// This function returns a pointer to the struct owner, given a struct pointer as input
pub fn get_struct_owner_fn(module: &mut Module) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::GetStructOwner.name().to_owned())
        .func_body();

    let struct_ptr = module.locals.add(ValType::I32);

    builder
        .local_get(struct_ptr)
        .i32_const(32)
        .binop(BinaryOp::I32Sub);

    function.finish(vec![struct_ptr], &mut module.funcs)
}

/// This function loops over all the mutably borrowed dynamic fields and save their changes into
/// the storage's cache.
///
/// After that, it flushes the cache, commiting the changes made for common storage structures (the
/// changes are saved in the `Ret` function that obtains them) and dynamic fields.
///
/// This function is executed after the the entrypoint called function finishes.
pub fn add_commit_changes_to_storage_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    dynamic_fields_global_variables: &Vec<(GlobalId, IntermediateType)>,
) -> Result<FunctionId, RuntimeFunctionError> {
    let mut function = FunctionBuilder::new(&mut module.types, &[], &[]);
    let mut builder = function
        .name(RuntimeFunction::CommitChangesToStorage.name().to_owned())
        .func_body();

    let (storage_flush_cache, _) = storage_flush_cache(module);

    // If we have dynamic fields to process, we put the code to process them.
    if !dynamic_fields_global_variables.is_empty() {
        let get_struct_owner_fn = RuntimeFunction::GetStructOwner.get(module, None, None)?;
        let get_id_bytes_ptr_fn =
            RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx), None)?;
        let write_object_slot_fn =
            RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx), None)?;
        let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx), None)?;

        let owner_ptr = module.locals.add(ValType::I32);
        for (dynamic_field_ptr, itype) in dynamic_fields_global_variables {
            let save_struct_into_storage_fn = RuntimeFunction::EncodeAndSaveInStorage.get_generic(
                module,
                compilation_ctx,
                Some(runtime_error_data),
                &[itype],
            )?;

            builder.block(None, |block| {
                let block_id = block.id();
                // The global id can be declares but never filled because the path that the code
                // took never called the borrow_mut function. In that case it will have assigned te
                // -1 value, we skip processing it
                block
                    .global_get(*dynamic_field_ptr)
                    .i32_const(-1)
                    .binop(BinaryOp::I32Eq)
                    .br_if(block_id);

                // Calculate the destiny slot

                // Put in stack the parent address
                block
                    .global_get(*dynamic_field_ptr)
                    .call(get_struct_owner_fn)
                    .local_tee(owner_ptr);

                // If the owner id is all zeroes, means the struct has no owner, and probably was
                // deleted from storage, so we skip the save
                block.i32_const(32).call(is_zero_fn).br_if(block_id);

                let slot_ptr = module.locals.add(ValType::I32);

                block
                    .i32_const(32)
                    .call(compilation_ctx.allocator)
                    .local_set(slot_ptr);

                // Put in the stack the field id
                block
                    .local_get(owner_ptr)
                    .global_get(*dynamic_field_ptr)
                    .call(get_id_bytes_ptr_fn)
                    .local_get(slot_ptr)
                    .call(write_object_slot_fn);

                // Save struct changes
                block
                    .global_get(*dynamic_field_ptr)
                    .local_get(slot_ptr)
                    .call(save_struct_into_storage_fn);
            });
        }
    }

    builder.i32_const(1).call(storage_flush_cache);

    Ok(function.finish(vec![], &mut module.funcs))
}

/// Emits a WASM function that maintains a 32-byte storage slot accumulator for DELETE flows.
///
/// Behavior:
/// * If `slot_offset + field_size` exceeds 32, it performs the delete-specific transition:
///   wipes the current slot data to zero and advances to the next slot.
///   Then sets `slot_offset = field_size`.
/// * Otherwise, it simply accumulates: `slot_offset += field_size`.
///
/// # WASM Function Arguments
/// * `slot_ptr` - (i32): Pointer to the slot.
/// * `slot_offset` - (i32): Offset in the current slot.
/// * `field_size` - (i32): Size of the field in bytes.
///
/// # WASM Function Returns
/// * (i32): The new slot offset after advancing or accumulating.
pub fn accumulate_or_advance_slot_delete(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let (storage_cache_fn, _) = storage_cache_bytes32(module);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx), None)?;

    Ok(build_accumulate_or_advance_slot(
        module,
        "accumulate_or_advance_slot_delete",
        move |then, slot_ptr| {
            // Wipe the slot
            then.local_get(slot_ptr)
                .i32_const(DATA_ZERO_OFFSET)
                .call(storage_cache_fn);

            // Advance the slot pointer
            then.local_get(slot_ptr)
                .call(next_slot_fn)
                .local_set(slot_ptr);
        },
    ))
}

/// Emits a WASM function that maintains a 32-byte storage slot accumulator for READ flows.
///
/// Behavior:
/// * If `slot_offset + field_size` exceeds 32, it advances to the next slot and loads the slot
///   data from storage (into the standard data buffer), then sets `slot_offset = field_size`.
/// * Otherwise, it accumulates: `slot_offset += field_size`.
///
/// # Arguments
/// * `slot_ptr` - i32: Pointer to the slot.
/// * `slot_offset` - i32: Offset in the current slot.
/// * `field_size` - i32: Size of the field in bytes.
///
/// # Returns
/// * i32: The new slot offset after advancing or accumulating.
pub fn accumulate_or_advance_slot_read(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let (storage_load, _) = storage_load_bytes32(module);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx), None)?;

    Ok(build_accumulate_or_advance_slot(
        module,
        "accumulate_or_advance_slot_read",
        move |then, slot_ptr| {
            // Advance the slot
            then.local_get(slot_ptr)
                .call(next_slot_fn)
                .local_tee(slot_ptr);

            // Load the slot data from storage
            then.i32_const(DATA_SLOT_DATA_PTR_OFFSET).call(storage_load);
        },
    ))
}

/// Emits a WASM function that maintains a 32-byte storage slot accumulator for WRITE flows.
///
/// Behavior:
/// * If `slot_offset + field_size` exceeds 32, it caches the current slot to storage,
///   clears the data buffer, advances to the next slot, and sets `slot_offset = field_size`.
/// * Otherwise, it accumulates: `slot_offset += field_size`.
///
/// # WASM Function Arguments
/// * `slot_ptr` - (i32): Pointer to the slot.
/// * `slot_offset` - (i32): Offset in the current slot.
/// * `field_size` - (i32): Size of the field in bytes.
///
/// # WASM Function Returns
/// * (i32): The new slot offset after advancing or accumulating.
pub fn accumulate_or_advance_slot_write(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> Result<FunctionId, RuntimeFunctionError> {
    let (storage_cache_fn, _) = storage_cache_bytes32(module);
    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx), None)?;

    Ok(build_accumulate_or_advance_slot(
        module,
        "accumulate_or_advance_slot_write",
        move |then, slot_ptr| {
            // Cache the slot data to storage
            then.local_get(slot_ptr)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(storage_cache_fn);

            // Wipe the slot data so we can write on it safely
            then.i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(0)
                .i32_const(32)
                .memory_fill(compilation_ctx.memory_id);

            // Advance the slot pointer
            then.local_get(slot_ptr)
                .call(next_slot_fn)
                .local_set(slot_ptr);
        },
    ))
}

/// Internal template used by the read/write/delete variants.
///
/// This helper encapsulates the common logic of deciding whether to accumulate within the current
/// 32-byte slot or advance to the next slot. It accepts a small closure (`mode_builder`) that emits
/// the mode-specific instructions executed when an advance is required (the "then" arm).
///
/// Template behavior:
/// * If `slot_offset + field_size > 32`:
///   * Executes the `mode_builder` closure to perform mode-specific actions (e.g., cache/clear/advance).
///   * Sets `slot_offset = field_size`.
/// * Else:
///   * Sets `slot_offset = slot_offset + field_size`.
///
/// The generated function has signature:
///   (slot_ptr: i32, slot_offset: i32, field_size: i32) -> i32 (new slot_offset)
fn build_accumulate_or_advance_slot<F>(
    module: &mut Module,
    name: &str,
    mode_builder: F,
) -> FunctionId
where
    F: FnOnce(&mut walrus::InstrSeqBuilder, walrus::LocalId),
{
    let mut function = FunctionBuilder::new(
        &mut module.types,
        &[ValType::I32, ValType::I32, ValType::I32],
        &[ValType::I32],
    );
    let mut builder = function.name(name.to_owned()).func_body();

    let slot_ptr = module.locals.add(ValType::I32);
    let slot_offset = module.locals.add(ValType::I32);
    let field_size = module.locals.add(ValType::I32);

    builder
        .local_get(slot_offset)
        .local_get(field_size)
        .binop(BinaryOp::I32Add)
        .i32_const(32)
        .binop(BinaryOp::I32GtU)
        .if_else(
            None,
            |then| {
                // Delegate the mode-specific logic
                mode_builder(then, slot_ptr);

                then.local_get(field_size).local_set(slot_offset);
            },
            |else_| {
                // Increment the slot_offset by the field size
                else_
                    .local_get(slot_offset)
                    .local_get(field_size)
                    .binop(BinaryOp::I32Add)
                    .local_set(slot_offset);
            },
        );

    builder.local_get(slot_offset);

    function.finish(vec![slot_ptr, slot_offset, field_size], &mut module.funcs)
}
/// Commits changes of storage objests into the storage cache.
///
/// # WASM Function Arguments
/// * `struct_ptr_ref` - (i32): pointer to a mutable reference of a storage struct
pub fn cache_storage_object_changes(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    runtime_error_data: &mut RuntimeErrorData,
    itype: &IntermediateType,
) -> Result<FunctionId, RuntimeFunctionError> {
    let name = RuntimeFunction::CacheStorageObjectChanges
        .get_generic_function_name(compilation_ctx, &[itype])?;
    if let Some(function) = module.funcs.by_name(&name) {
        return Ok(function);
    }

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let get_struct_owner_fn = RuntimeFunction::GetStructOwner.get(module, None, None)?;
    let locate_struct_fn =
        RuntimeFunction::LocateStructSlot.get(module, Some(compilation_ctx), None)?;
    let save_in_slot_fn = RuntimeFunction::EncodeAndSaveInStorage.get_generic(
        module,
        compilation_ctx,
        Some(runtime_error_data),
        &[itype],
    )?;
    let check_and_delete_struct_tto_fields_fn = RuntimeFunction::CheckAndDeleteStructTtoFields
        .get_generic(module, compilation_ctx, Some(runtime_error_data), &[itype])?;

    // Arguments
    let struct_ptr_ref = module.locals.add(ValType::I32);

    // Locals
    let struct_ptr = module.locals.add(ValType::I32);
    builder
        .local_get(struct_ptr_ref)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
        .local_tee(struct_ptr);

    // Compute the slot where the struct will be saved
    builder.call(locate_struct_fn);

    // Check if the object owner is zero
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx), None)?;
    builder
        .local_get(struct_ptr)
        .call(get_struct_owner_fn)
        .i32_const(32)
        .call(is_zero_fn);

    builder.if_else(
        None,
        |_| {
            // If the object owner is zero, it means the object was deleted and we don't need to save it
        },
        |else_| {
            // Copy the slot number to a local to avoid overwriting it later
            let slot_ptr = module.locals.add(ValType::I32);
            else_
                .i32_const(32)
                .call(compilation_ctx.allocator)
                .local_tee(slot_ptr)
                .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                .i32_const(32)
                .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

            // Call the function to delete any recently tto objects within the struct
            // This needed when pushing objects with the key ability into a vector field of a struct
            else_
                .local_get(struct_ptr)
                .call(check_and_delete_struct_tto_fields_fn);

            // Save the struct in the slot
            else_
                .local_get(struct_ptr)
                .local_get(slot_ptr)
                .call(save_in_slot_fn);
        },
    );

    Ok(function.finish(vec![struct_ptr_ref], &mut module.funcs))
}

// The expected slot values were calculated using Remix to ensure the tests are correct.
#[cfg(test)]
mod tests {
    use crate::test_compilation_context;
    use crate::test_tools::{
        build_module, get_linker_with_native_keccak256, setup_wasmtime_module,
    };
    use alloy_primitives::U256;
    use rstest::rstest;
    use std::str::FromStr;
    use walrus::{FunctionBuilder, ValType};

    use super::*;

    #[rstest]
    #[case(
        U256::from(1),
        U256::from(2),
        U256::from_str(
            "98521912898304110675870976153671229506380941016514884467413255631823579132687"
        ).unwrap()
    )]
    #[case(
        U256::from(1),
        U256::from(3),
        U256::from_str(
            "56988696150268759067033853745049141362335364605175666696514897554729450063371"
    ).unwrap()
    )]
    #[case(
        U256::from(1),
        U256::from(123456789),
        U256::from_str(
            "66492595055558910473828628519319372113473818625668867548228543292688569385097"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        U256::from(2),
        U256::from_str(
            "46856049987324987851654180578118835177937932377897439695260177228387632849548"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        U256::from(3),
        U256::from_str(
            "61684305963762951884865369267618438865725240706238913880678826931473020346819"
    ).unwrap()
    )]
    fn test_derive_mapping_slot(#[case] slot: U256, #[case] key: U256, #[case] expected: U256) {
        let (mut module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(Some(64));

        let slot_ptr = module.locals.add(ValType::I32);
        let key_ptr = module.locals.add(ValType::I32);
        let result_ptr = module.locals.add(ValType::I32);

        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        let mut func_body = builder.func_body();

        // Allocate memory for the result
        func_body
            .i32_const(32)
            .call(allocator_func)
            .local_set(result_ptr);

        let ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        // Call derive_mapping_slot with the proper arguments
        func_body
            .local_get(slot_ptr)
            .local_get(key_ptr)
            .local_get(result_ptr)
            .call(derive_mapping_slot(&mut module, &ctx));

        // Return the result pointer
        func_body.local_get(result_ptr);

        let function = builder.finish(vec![slot_ptr, key_ptr], &mut module.funcs);
        module.exports.add("test_fn", function);

        let linker = get_linker_with_native_keccak256();

        let data = [slot.to_be_bytes::<32>(), key.to_be_bytes::<32>()].concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint.call(&mut store, (0, 32)).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_bytes = vec![0; 32];
        memory
            .read(&mut store, pointer as usize, &mut result_bytes)
            .unwrap();

        let result = U256::from_be_bytes::<32>(result_bytes.try_into().unwrap());

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        U256::from(1),
        U256::from(2),
        U256::from(4),
        U256::from_str(
            "23991499908108302765562531213920885141500505546388542086856722761454457053429"
        ).unwrap()
    )]
    #[case(
        U256::from(1),
        U256::from(5),
        U256::from(21),
        U256::from_str(
            "67151859839340103677100435873946963192465517128770968255452291644285690915775"
        ).unwrap()
    )]
    #[case(
        U256::from(2),
        U256::from(7),
        U256::from(28),
        U256::from_str(
            "70122961159721460691158963782174993504655102344268525554192115423808014779926"
        ).unwrap()
    )]
    fn test_derive_nested_mapping_slot(
        #[case] slot: U256,
        #[case] outer_key: U256,
        #[case] inner_key: U256,
        #[case] expected: U256,
    ) {
        let (mut module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(Some(96));

        let slot_ptr = module.locals.add(ValType::I32);
        let outer_key_ptr = module.locals.add(ValType::I32);
        let inner_key_ptr = module.locals.add(ValType::I32);

        // Allocate memory for the result
        let nested_mapping_slot_ptr = module.locals.add(ValType::I32);
        let result_ptr = module.locals.add(ValType::I32);

        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        let mut func_body = builder.func_body();

        func_body
            .i32_const(32)
            .call(allocator_func)
            .local_set(result_ptr);

        let ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        // Call derive_mapping_slot with the proper arguments
        func_body
            .local_get(slot_ptr)
            .local_get(outer_key_ptr)
            .local_get(nested_mapping_slot_ptr)
            .call(derive_mapping_slot(&mut module, &ctx));

        func_body
            .local_get(nested_mapping_slot_ptr)
            .local_get(inner_key_ptr)
            .local_get(result_ptr)
            .call(derive_mapping_slot(&mut module, &ctx));

        func_body.local_get(result_ptr);
        let function = builder.finish(
            vec![slot_ptr, outer_key_ptr, inner_key_ptr],
            &mut module.funcs,
        );
        module.exports.add("test_fn", function);

        let linker = get_linker_with_native_keccak256();

        let data = [
            slot.to_be_bytes::<32>(),
            outer_key.to_be_bytes::<32>(),
            inner_key.to_be_bytes::<32>(),
        ]
        .concat();
        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint.call(&mut store, (0, 32, 64)).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_bytes = vec![0; 32];
        memory
            .read(&mut store, pointer as usize, &mut result_bytes)
            .unwrap();

        let result = U256::from_be_bytes::<32>(result_bytes.try_into().unwrap());

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        U256::from(2),
        0_u32,
        4_u32,
        U256::from_str(
            "29102676481673041902632991033461445430619272659676223336789171408008386403022"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        1_u32,
        4_u32,
        U256::from_str(
            "29102676481673041902632991033461445430619272659676223336789171408008386403022"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        7_u32,
        4_u32,
        U256::from_str(
            "29102676481673041902632991033461445430619272659676223336789171408008386403022"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        8_u32,
        4_u32,
        U256::from_str(
            "29102676481673041902632991033461445430619272659676223336789171408008386403023"
    ).unwrap()
    )]
    #[case(
        U256::from(3),
        0_u32,
        36_u32,
        U256::from_str(
            "87903029871075914254377627908054574944891091886930582284385770809450030037083"
    ).unwrap()
    )]
    #[case(
        U256::from(3),
        1_u32,
        36_u32,
        U256::from_str(
            "87903029871075914254377627908054574944891091886930582284385770809450030037085"
    ).unwrap()
    )]
    #[case(
        U256::from(3),
        2_u32,
        36_u32,
        U256::from_str(
            "87903029871075914254377627908054574944891091886930582284385770809450030037087"
    ).unwrap()
    )]
    #[should_panic]
    #[case(
        U256::from(3),
        2_u32,
        0_u32,
        U256::from_str(
            "87903029871075914254377627908054574944891091886930582284385770809450030037087"
    ).unwrap()
    )]
    fn test_derive_dyn_array_slot(
        #[case] header_slot: U256,
        #[case] element_index: u32,
        #[case] element_size: u32,
        #[case] expected: U256,
    ) {
        let (mut module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(Some(40)); // slot (32 bytes) + index (4 bytes) + elem_size (4 bytes)

        let slot_ptr = module.locals.add(ValType::I32);
        let index = module.locals.add(ValType::I32);
        let elem_size = module.locals.add(ValType::I32);
        let result_ptr = module.locals.add(ValType::I32);

        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        let mut func_body = builder.func_body();

        func_body
            .i32_const(32)
            .call(allocator_func)
            .local_set(result_ptr);

        let ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        func_body
            .local_get(slot_ptr)
            .local_get(index)
            .local_get(elem_size)
            .local_get(result_ptr)
            .call(derive_dyn_array_slot(&mut module, &ctx).unwrap());

        func_body.local_get(result_ptr);
        let function = builder.finish(vec![slot_ptr, index, elem_size], &mut module.funcs);
        module.exports.add("test_fn", function);

        let linker = get_linker_with_native_keccak256();

        let data = [
            header_slot.to_be_bytes::<32>().to_vec(),
            element_index.to_le_bytes().to_vec(),
            element_size.to_le_bytes().to_vec(),
        ]
        .concat();

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint
            .call(&mut store, (0, element_index as i32, element_size as i32))
            .unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_bytes = vec![0; 32];
        memory
            .read(&mut store, pointer as usize, &mut result_bytes)
            .unwrap();

        let result = U256::from_be_bytes::<32>(result_bytes.try_into().unwrap());

        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(
        U256::from(2),
        0_u32,
        1_u32,
        4_u32,
        U256::from_str(
            "12072469696963966767691700411905649679726912322096881580412568241040270596576"
    ).unwrap()
    )]
    #[case(
        U256::from(2),
        1_u32,
        1_u32,
        4_u32,
        U256::from_str(
            "21317519515597955722743988462724083255677628835556397468395520694449519796017"
    ).unwrap()
    )]
    fn test_derive_nested_dyn_array_slot(
        #[case] header_slot: U256,
        #[case] element_outer_index: u32,
        #[case] element_inner_index: u32,
        #[case] element_size: u32,
        #[case] expected: U256,
    ) {
        // slot (32 bytes) + outer_index (4 bytes) + inner_index (4 bytes) + elem_size (4 bytes)
        let (mut module, allocator_func, memory_id, calldata_reader_pointer_global) =
            build_module(Some(44));

        let slot_ptr = module.locals.add(ValType::I32);
        let outer_index = module.locals.add(ValType::I32);
        let inner_index = module.locals.add(ValType::I32);
        let elem_size = module.locals.add(ValType::I32);
        let array_header_size = module.locals.add(ValType::I32);
        let result_ptr = module.locals.add(ValType::I32);

        let mut builder = FunctionBuilder::new(
            &mut module.types,
            &[ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            &[ValType::I32],
        );
        let mut func_body = builder.func_body();

        func_body
            .i32_const(32)
            .call(allocator_func)
            .local_set(result_ptr);

        func_body // the header of the array occupies exactly 1 slot i.e. 32 bytes
            .i32_const(32)
            .local_set(array_header_size);

        let ctx =
            test_compilation_context!(memory_id, allocator_func, calldata_reader_pointer_global);

        // Call derive_dyn_array_slot_for_index with the proper arguments
        func_body
            .local_get(slot_ptr)
            .local_get(outer_index)
            .local_get(array_header_size)
            .local_get(result_ptr)
            .call(derive_dyn_array_slot(&mut module, &ctx).unwrap());

        func_body
            .local_get(result_ptr)
            .local_get(inner_index)
            .local_get(elem_size)
            .local_get(result_ptr)
            .call(derive_dyn_array_slot(&mut module, &ctx).unwrap());

        func_body.local_get(result_ptr);
        let function = builder.finish(
            vec![slot_ptr, outer_index, inner_index, elem_size],
            &mut module.funcs,
        );
        module.exports.add("test_fn", function);

        let linker = get_linker_with_native_keccak256();

        let data = [
            header_slot.to_be_bytes::<32>().to_vec(),
            element_outer_index.to_le_bytes().to_vec(),
            element_inner_index.to_le_bytes().to_vec(),
            element_size.to_le_bytes().to_vec(),
        ]
        .concat();

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint
            .call(
                &mut store,
                (
                    0,
                    element_outer_index as i32,
                    element_inner_index as i32,
                    element_size as i32,
                ),
            )
            .unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_bytes = vec![0; 32];
        memory
            .read(&mut store, pointer as usize, &mut result_bytes)
            .unwrap();

        let result = U256::from_be_bytes::<32>(result_bytes.try_into().unwrap());

        assert_eq!(result, expected);
    }
}
