//! Initializes the data segment for the module

use alloy_primitives::U256;
use walrus::{ConstExpr, DataKind, MemoryId, Module, ir::Value};

/// u256 one in little endian. This is used to add it to the pointer that contains a slot number
/// when moving to the next storage slot when reading/writing it.
pub const DATA_U256_ONE_OFFSET: i32 = 0;

/// Used to hold the slot data currently being read/written. We actually don't initialize this
/// because it is not needed, we just set the offset to mark it as reseverd space.
pub const DATA_SLOT_DATA_PTR_OFFSET: i32 = 32;

/// Slot 0 of the storage used for the objects mapping
pub const DATA_OBJECTS_SLOT_OFFSET: i32 = 64;

/// Amount of memory reserved starting from offset 0.
///
/// # WARNING
/// This value must be kept in sync to correctly initialize the memory allocator
/// at the proper offset.
pub const TOTAL_RESERVED_MEMORY: i32 = 96;

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
}
