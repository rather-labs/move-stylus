//! Initializes the data segment for the module

use alloy_primitives::U256;
use walrus::{ConstExpr, DataKind, MemoryId, Module, ir::Value};

/// Reserved empty memory
pub const DATA_ZERO_OFFSET: i32 = 0;

/// u256 one in little endian. This is used to add it to the pointer that contains a slot number
/// when moving to the next storage slot when reading/writing it.
pub const DATA_U256_ONE_OFFSET: i32 = 32;

/// Used to hold the slot data currently being read/written. We actually don't initialize this
/// because it is not needed, we just set the offset to mark it as reseverd space.
pub const DATA_SLOT_DATA_PTR_OFFSET: i32 = 64;

/// Slot 0 of the storage used for the objects mapping
pub const DATA_OBJECTS_SLOT_OFFSET: i32 = 96;

/// When calculating slots numbers, we save them here
pub const DATA_OBJECTS_MAPPING_SLOT_NUMBER_OFFSET: i32 = 128;

/// This is the shared objects mapping key. It is a u256 containing the number 1
pub const DATA_SHARED_OBJECTS_KEY_OFFSET: i32 = 160;

/// This is the frozen objects mapping key. It is a u256 containing the number 2
pub const DATA_FROZEN_OBJECTS_KEY_OFFSET: i32 = 192;

/// When searching for the object's owner in the objects mapping, we will use this piece of
/// memory to save the owner where the object was found (an address, key 1 for shared and
/// key 2 for frozen).
pub const DATA_STORAGE_OBJECT_OWNER_OFFSET: i32 = 224;

/// Used to store the pointer to the abort message when translating an Abort instruction.
/// The memory layout is structured as: [length, data], where the first byte represent the length of the message.
pub const DATA_ABORT_MESSAGE_PTR_OFFSET: i32 = 256;

/// When searching for the enum's storage size, we will use this piece of memory to save the size of the enum.
/// It takes 128 bytes (32 * 4 bytes) to store the size of the enum for each offset.
pub const DATA_ENUM_STORAGE_SIZE_OFFSET: i32 = 260;

/// Stores the length and pointer to the raw calldata
/// 8 bytes: [data_length][data_pointer]
pub const DATA_CALLDATA_OFFSET: i32 = 396;

/// Amount of memory reserved starting from offset 0.
///
/// # WARNING
/// This value must be kept in sync to correctly initialize the memory allocator
/// at the proper offset.
pub const TOTAL_RESERVED_MEMORY: i32 = 404;

/// Initializes the module's data segment.
pub fn setup_data_segment(module: &mut Module, memory_id: MemoryId) {
    // DATA_U256_ONE_OFFSET initialization
    let data = U256::from(1).to_le_bytes_vec();
    module.data.add(
        DataKind::Active {
            memory: memory_id,
            offset: ConstExpr::Value(Value::I32(DATA_U256_ONE_OFFSET)),
        },
        data,
    );

    let data = U256::from(1).to_be_bytes_vec();
    module.data.add(
        DataKind::Active {
            memory: memory_id,
            offset: ConstExpr::Value(Value::I32(DATA_SHARED_OBJECTS_KEY_OFFSET)),
        },
        data,
    );

    let data = U256::from(2).to_be_bytes_vec();
    module.data.add(
        DataKind::Active {
            memory: memory_id,
            offset: ConstExpr::Value(Value::I32(DATA_FROZEN_OBJECTS_KEY_OFFSET)),
        },
        data,
    );
}
