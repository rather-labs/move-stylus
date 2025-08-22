use super::RuntimeFunction;
use crate::data::{
    DATA_FROZEN_OBJECTS_KEY_OFFSET, DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET,
    DATA_OBJECTS_SLOT_OFFSET, DATA_SHARED_OBJECTS_KEY_OFFSET, DATA_SLOT_DATA_PTR_OFFSET,
    DATA_STORAGE_OBJECT_OWNER_OFFSET,
};
use crate::hostio::host_functions::{self, storage_cache_bytes32, storage_load_bytes32, tx_origin};
use crate::storage::encoding::{
    add_encode_and_save_into_storage_struct_instructions,
    add_read_and_decode_storage_struct_instructions,
};
use crate::translation::intermediate_types::IntermediateType;
use crate::translation::intermediate_types::heap_integers::IU256;
use crate::wasm_builder_extensions::WasmBuilderExtension;
use crate::{CompilationContext, data::DATA_U256_ONE_OFFSET};
use crate::{get_generic_function_name, storage};
use walrus::{
    FunctionBuilder, FunctionId, Module, ValType,
    ir::{BinaryOp, LoadKind, MemArg, StoreKind},
};

/// Looks for an struct inside the objects mappings. The objects mappings follows the solidity notation:
/// mapping(bytes32 => mapping(bytes32 => T)) public moveObjects;
///
/// Where:
/// - The outer mapping key is the id of the owner (could be an address or object id).
/// - The inner mapping key is the object id itself.
/// - The value is the encoded structure.
///
/// The lookup is done in the following order:
/// - In the signer's owned objects (key is the signer's address).
/// - In the shared objects key (1)
/// - In the frozen objects key (2)
///
/// If no data is found an unrechable error is thrown. Otherwise the slot number to reconstruct the
/// struct is written in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET.
///
/// When the data is found, the owner's ID is written in DATA_STORAGE_OBJECT_OWNER_OFFSET
///
/// # Arguments
/// - object id
pub fn locate_storage_data(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    // Runtime functions
    let is_zero_fn = RuntimeFunction::IsZero.get(module, Some(compilation_ctx));
    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));

    // Host functions
    let (tx_origin, _) = tx_origin(module);
    let (storage_load, _) = storage_load_bytes32(module);

    // Function declaration
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::LocateStorageData.name().to_owned())
        .func_body();

    // Arguments
    let uid_ptr = module.locals.add(ValType::I32);
    let search_frozen = module.locals.add(ValType::I32);

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

    builder.block(None, |block| {
        let exit_block = block.id();

        // ==
        // Signer's objects
        // ==
        block
            .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
            .local_get(uid_ptr)
            .call(write_object_slot_fn);

        // Load data from slot
        block
            .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_load);

        // Check if it is empty (all zeroes)
        block
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .i32_const(32)
            .call(is_zero_fn)
            .negate()
            .br_if(exit_block);

        // ==
        // Shared objects
        // ==

        // Copy the shared objects key to the owners offset
        block
            .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
            .i32_const(DATA_SHARED_OBJECTS_KEY_OFFSET)
            .i32_const(32)
            .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

        block
            .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
            .local_get(uid_ptr)
            .call(write_object_slot_fn);

        // Load data from slot
        block
            .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .call(storage_load);

        // Check if it is empty (all zeroes)
        block
            .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
            .i32_const(32)
            .call(is_zero_fn)
            .negate()
            .br_if(exit_block);

        // ==
        // Frozen objects
        // ==
        // Copy the frozen objects key to the owners offset
        block.block(None, |frozen_block| {
            let exit_frozen_block = frozen_block.id();
            frozen_block
                .local_get(search_frozen)
                .i32_const(0)
                .binop(BinaryOp::I32Eq)
                .br_if(exit_frozen_block);

            frozen_block
                .i32_const(DATA_STORAGE_OBJECT_OWNER_OFFSET)
                .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
                .i32_const(32)
                .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

            frozen_block
                .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
                .local_get(uid_ptr)
                .call(write_object_slot_fn);

            // Load data from slot
            frozen_block
                .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(storage_load);

            // Check if it is empty (all zeroes)
            frozen_block
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(32)
                .call(is_zero_fn)
                .negate()
                .br_if(exit_block);
        });

        // If we get here means the object was not found
        block.unreachable();
    });

    function.finish(vec![uid_ptr, search_frozen], &mut module.funcs)
}

/// Computes the storage slot number where the struct should be persisted.
///
/// When working with a struct in memory that has the `key` ability,
/// once processing is complete, its storage slot must be calculated
/// so the changes can be saved.
///
/// The slot number is written in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET.
///
/// # Arguments
/// - struct_ptr - pointer to the struct
pub fn locate_struct_slot(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::LocateStructSlot.name().to_owned())
        .func_body();

    let write_object_slot_fn = RuntimeFunction::WriteObjectSlot.get(module, Some(compilation_ctx));
    let get_id_bytes_ptr_fn = RuntimeFunction::GetIdBytesPtr.get(module, Some(compilation_ctx));
    let struct_ptr = module.locals.add(ValType::I32);

    // Obtain this object's owner
    builder
        .local_get(struct_ptr)
        .i32_const(32)
        .binop(BinaryOp::I32Sub);

    // Get the pointer to the 32 bytes holding the data of the id
    builder.local_get(struct_ptr).call(get_id_bytes_ptr_fn);

    // Compute the slot where it should be saved
    builder.call(write_object_slot_fn);

    function.finish(vec![struct_ptr], &mut module.funcs)
}

/// Calculates the slot from the slot mapping
pub fn write_object_slot(module: &mut Module, compilation_ctx: &CompilationContext) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function
        .name(RuntimeFunction::WriteObjectSlot.name().to_owned())
        .func_body();

    let uid_ptr = module.locals.add(ValType::I32);
    let owner_ptr = module.locals.add(ValType::I32);

    // Calculate the slot address
    let derive_slot_fn = RuntimeFunction::DeriveMappingSlot.get(module, Some(compilation_ctx));

    // Derive the slot for the first mapping
    builder
        .i32_const(DATA_OBJECTS_SLOT_OFFSET)
        .local_get(owner_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .call(derive_slot_fn);

    // Derive slot for ther second mapping
    builder
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .local_get(uid_ptr)
        .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
        .call(derive_slot_fn);

    function.finish(vec![owner_ptr, uid_ptr], &mut module.funcs)
}

pub fn storage_next_slot_function(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function
        .name(RuntimeFunction::StorageNextSlot.name().to_owned())
        .func_body();

    let slot_ptr = module.locals.add(ValType::I32);

    let swap_256_fn = RuntimeFunction::SwapI256Bytes.get(module, Some(compilation_ctx));
    let add_u256_fn = RuntimeFunction::HeapIntSum.get(module, Some(compilation_ctx));

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

    function.finish(vec![slot_ptr], &mut module.funcs)
}

// This function returns a pointer to the 32 bytes holding the data of the id, given a struct pointer as input
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
/// - for value types, h pads the value to 32 bytes in the same way as when storing the value in memory.
/// - for strings and byte arrays, h(k) is just the unpadded data.
///
/// Arguments:
/// - `mapping_slot_ptr`: pointer to the mapping slot (32 bytes)
/// - `key_ptr`: pointer to the key (32 bytes)
/// - `derived_slot_ptr`: pointer to the derived slot (32 bytes)
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

    // Allocate memory for the hash data
    let data_ptr = module.locals.add(ValType::I32);

    builder
        .i32_const(64) // For now this is always 64 bytes as we are not dealing with dynamic keys yet
        .call(compilation_ctx.allocator)
        .local_set(data_ptr);

    builder
        .local_get(data_ptr)
        .local_get(key_ptr)
        .i32_const(32) // copy 32 bytes, for now fixed size
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    builder
        .local_get(data_ptr)
        .i32_const(32)
        .binop(BinaryOp::I32Add) // data_ptr + 32
        .local_get(mapping_slot_ptr)
        .i32_const(32) // copy 32 bytes
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    // Hash the data, this is the mapping slot we are looking for -> v = keccak256(h(k) . p)
    builder
        .local_get(data_ptr)
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
/// Parameters:
/// - `array_slot_ptr`: A pointer to the u256 slot `p`, which is the header slot of the array.
/// - `elem_index_ptr`: A pointer to the u32 value representing the element's index in the array (little-endian).
/// - `elem_size_ptr`: A pointer to the u32 value representing the size of each element in bytes (little-endian).
///
/// The computed u256 slot value for the element, in big-endian format, is stored at `derived_elem_slot_ptr`.
pub fn derive_dyn_array_slot(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
) -> FunctionId {
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
    let elem_index_ptr = module.locals.add(ValType::I32);
    let elem_size_ptr = module.locals.add(ValType::I32);
    let derived_elem_slot_ptr = module.locals.add(ValType::I32);

    let (native_keccak, _) = host_functions::native_keccak256(module);
    let swap_i32_bytes_fn = RuntimeFunction::SwapI32Bytes.get(module, None);

    // Guard: check elem_size is greater than 0
    builder
        .local_get(elem_size_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
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
        .local_get(elem_size_ptr)
        .load(
            compilation_ctx.memory_id,
            LoadKind::I32 { atomic: false },
            MemArg {
                align: 0,
                offset: 0,
            },
        )
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
            // Step 1: Load the index (u32)
            then.local_get(elem_index_ptr).load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // Step 2: Load the element size and compute divisor = floor(32 / elem_size)
            then.i32_const(32)
                .local_get(elem_size_ptr)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
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
            // Step 1: Load the index (u32)
            else_.local_get(elem_index_ptr).load(
                compilation_ctx.memory_id,
                LoadKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

            // Step 2: Compute slots_per_element = (elem_size + 31) / 32
            else_
                .local_get(elem_size_ptr)
                .load(
                    compilation_ctx.memory_id,
                    LoadKind::I32 { atomic: false },
                    MemArg {
                        align: 0,
                        offset: 0,
                    },
                )
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
    let elem_offset_32 = elem_size_ptr;
    builder.local_set(elem_offset_32);

    // Repurpose elem_index_ptr to allocate and hold the offset as U256
    let elem_offset_256_ptr = elem_index_ptr;
    builder
        .i32_const(32)
        .call(compilation_ctx.allocator)
        .local_set(elem_offset_256_ptr)
        .local_get(elem_offset_256_ptr)
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
    builder.local_get(derived_elem_slot_ptr);
    builder
        .local_get(elem_offset_256_ptr)
        .local_get(base_slot_ptr);
    IU256::add(&mut builder, module, compilation_ctx); // add(base, offset) with overflow check
    builder // copy add(base, offset) result to #derived_elem_slot_ptr
        .i32_const(32)
        .memory_copy(compilation_ctx.memory_id, compilation_ctx.memory_id);

    function.finish(
        vec![
            array_slot_ptr,
            elem_index_ptr,
            elem_size_ptr,
            derived_elem_slot_ptr,
        ],
        &mut module.funcs,
    )
}

/// Generates a function that encodes and saves an specific struct into the storage.
///
/// Arguments:
/// - struct_ptr
/// - slot_ptr
pub fn add_save_struct_into_storage_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(RuntimeFunction::EncodeAndSaveInStorage.name(), &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    }

    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap();

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32, ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let struct_ptr = module.locals.add(ValType::I32);
    let slot_ptr = module.locals.add(ValType::I32);

    add_encode_and_save_into_storage_struct_instructions(
        module,
        &mut builder,
        compilation_ctx,
        struct_ptr,
        slot_ptr,
        &struct_,
        0,
    );

    function.finish(vec![struct_ptr, slot_ptr], &mut module.funcs)
}

// Generates a function that reads an specific struct from the storage.
//
// This function:
// 1. Locates the storage slot of the object.
// 2. Reads and decodes the struct from storage.
// 3. Returns a pointer to the in-memory representation of the struct.
//
// Arguments:
// - slot_ptr
//
// Returns:
// - struct_ptr
pub fn add_read_struct_from_storage_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name =
        get_generic_function_name(RuntimeFunction::DecodeAndReadFromStorage.name(), &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    }

    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap();

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[ValType::I32]);
    let mut builder = function.name(name).func_body();

    let slot_ptr = module.locals.add(ValType::I32);

    let (struct_ptr, _) = add_read_and_decode_storage_struct_instructions(
        module,
        &mut builder,
        compilation_ctx,
        slot_ptr,
        &struct_,
        false,
        0,
    );

    builder.local_get(struct_ptr);

    function.finish(vec![slot_ptr], &mut module.funcs)
}

/// Generates a function that deletes an object from storage.
///
/// This function:
/// 1. Validates the object is not frozen (frozen objects cannot be deleted).
/// 2. Locates the storage slot of the object.
/// 3. Clears the storage slot and any additional slots occupied by the struct fields.
/// 4. Flushes the cache to finalize the deletion.
///
/// Arguments:
/// - struct_ptr
pub fn add_delete_struct_from_storage_fn(
    module: &mut Module,
    compilation_ctx: &CompilationContext,
    itype: &IntermediateType,
) -> FunctionId {
    let name = get_generic_function_name(RuntimeFunction::DeleteFromStorage.name(), &[itype]);
    if let Some(function) = module.funcs.by_name(&name) {
        return function;
    };

    let struct_ = compilation_ctx
        .get_struct_by_intermediate_type(itype)
        .unwrap();

    let next_slot_fn = RuntimeFunction::StorageNextSlot.get(module, Some(compilation_ctx));
    let locate_struct_slot_fn =
        RuntimeFunction::LocateStructSlot.get(module, Some(compilation_ctx));
    let equality_fn = RuntimeFunction::HeapTypeEquality.get(module, Some(compilation_ctx));

    let (storage_cache, _) = storage_cache_bytes32(module);

    let mut function = FunctionBuilder::new(&mut module.types, &[ValType::I32], &[]);
    let mut builder = function.name(name).func_body();

    let slot_ptr = module.locals.add(ValType::I32);
    let struct_ptr = module.locals.add(ValType::I32);

    // Verify if the object is frozen; if not, continue.
    builder
        .local_get(struct_ptr)
        .i32_const(32)
        .binop(BinaryOp::I32Sub)
        .i32_const(DATA_FROZEN_OBJECTS_KEY_OFFSET)
        .i32_const(32)
        .call(equality_fn);

    builder.if_else(
        None,
        |then| {
            // Emit an unreachable if the object is frozen
            then.unreachable();
        },
        |else_| {
            // Calculate the object slot in the storage (saved in DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
            else_
                .local_get(struct_ptr)
                .call(locate_struct_slot_fn)
                .i32_const(DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET)
                .local_set(slot_ptr);

            // Wipe the slot data placeholder. We will use it to erase the slots in the storage
            else_
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .i32_const(0)
                .i32_const(32)
                .memory_fill(compilation_ctx.memory_id);

            // Wipe out the first slot
            else_
                .local_get(slot_ptr)
                .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                .call(storage_cache);

            // Loop through each field in the struct and clear the corresponding storage slots.
            let mut slot_used_bytes = 0;
            for field in struct_.fields.iter() {
                let field_size = storage::encoding::field_size(field, compilation_ctx);
                if slot_used_bytes + field_size > 32 {
                    else_
                        .local_get(slot_ptr)
                        .call(next_slot_fn)
                        .local_tee(slot_ptr)
                        .i32_const(DATA_SLOT_DATA_PTR_OFFSET)
                        .call(storage_cache);

                    slot_used_bytes = field_size;
                } else {
                    slot_used_bytes += field_size;
                }
            }
        },
    );

    function.finish(vec![struct_ptr], &mut module.funcs)
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
    use walrus::FunctionBuilder;

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
        let (mut module, allocator_func, memory_id) = build_module(Some(64));

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

        let ctx = test_compilation_context!(memory_id, allocator_func);

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
        let (mut module, allocator_func, memory_id) = build_module(Some(96));

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

        let ctx = test_compilation_context!(memory_id, allocator_func);

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
        #[case] slot: U256,
        #[case] index: u32,
        #[case] elem_size: u32,
        #[case] expected: U256,
    ) {
        let (mut module, allocator_func, memory_id) = build_module(Some(40)); // slot (32 bytes) + index (4 bytes) + elem_size (4 bytes)

        let slot_ptr = module.locals.add(ValType::I32);
        let index_ptr = module.locals.add(ValType::I32);
        let elem_size_ptr = module.locals.add(ValType::I32);
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

        let ctx = test_compilation_context!(memory_id, allocator_func);

        func_body
            .local_get(slot_ptr)
            .local_get(index_ptr)
            .local_get(elem_size_ptr)
            .local_get(result_ptr)
            .call(derive_dyn_array_slot(&mut module, &ctx));

        func_body.local_get(result_ptr);
        let function = builder.finish(vec![slot_ptr, index_ptr, elem_size_ptr], &mut module.funcs);
        module.exports.add("test_fn", function);

        let linker = get_linker_with_native_keccak256();

        let data = [
            slot.to_be_bytes::<32>().to_vec(),
            index.to_le_bytes().to_vec(),
            elem_size.to_le_bytes().to_vec(),
        ]
        .concat();

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint.call(&mut store, (0, 32, 36)).unwrap();
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
        #[case] slot: U256,
        #[case] outer_index: u32,
        #[case] inner_index: u32,
        #[case] elem_size: u32,
        #[case] expected: U256,
    ) {
        // slot (32 bytes) + outer_index (4 bytes) + inner_index (4 bytes) + elem_size (4 bytes)
        let (mut module, allocator_func, memory_id) = build_module(Some(44));

        let slot_ptr = module.locals.add(ValType::I32);
        let outer_index_ptr = module.locals.add(ValType::I32);
        let inner_index_ptr = module.locals.add(ValType::I32);
        let elem_size_ptr = module.locals.add(ValType::I32);
        let array_header_size_ptr = module.locals.add(ValType::I32);
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
            .i32_const(4)
            .call(allocator_func)
            .local_tee(array_header_size_ptr)
            .i32_const(32)
            .store(
                memory_id,
                StoreKind::I32 { atomic: false },
                MemArg {
                    align: 0,
                    offset: 0,
                },
            );

        let ctx = test_compilation_context!(memory_id, allocator_func);

        // Call derive_dyn_array_slot_for_index with the proper arguments
        func_body
            .local_get(slot_ptr)
            .local_get(outer_index_ptr)
            .local_get(array_header_size_ptr)
            .local_get(result_ptr)
            .call(derive_dyn_array_slot(&mut module, &ctx));

        func_body
            .local_get(result_ptr)
            .local_get(inner_index_ptr)
            .local_get(elem_size_ptr)
            .local_get(result_ptr)
            .call(derive_dyn_array_slot(&mut module, &ctx));

        func_body.local_get(result_ptr);
        let function = builder.finish(
            vec![slot_ptr, outer_index_ptr, inner_index_ptr, elem_size_ptr],
            &mut module.funcs,
        );
        module.exports.add("test_fn", function);

        let linker = get_linker_with_native_keccak256();

        let data = [
            slot.to_be_bytes::<32>().to_vec(),
            outer_index.to_le_bytes().to_vec(),
            inner_index.to_le_bytes().to_vec(),
            elem_size.to_le_bytes().to_vec(),
        ]
        .concat();

        let (_, instance, mut store, entrypoint) =
            setup_wasmtime_module(&mut module, data, "test_fn", Some(linker));

        let pointer: i32 = entrypoint.call(&mut store, (0, 32, 36, 40)).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let mut result_bytes = vec![0; 32];
        memory
            .read(&mut store, pointer as usize, &mut result_bytes)
            .unwrap();

        let result = U256::from_be_bytes::<32>(result_bytes.try_into().unwrap());

        assert_eq!(result, expected);
    }
}
