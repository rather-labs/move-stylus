//! Initializes the data segment for the module

use crate::abi_types::error_encoding::ERROR_SELECTOR;
use crate::error::RuntimeError;
use alloy_primitives::U256;
use std::collections::HashMap;
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

/// Struct to keep track of the encoded runtime errors and the next offset to write the next error.
pub struct RuntimeErrorData {
    next_offset: i32,
    errors: HashMap<RuntimeError, i32>,
}

impl Default for RuntimeErrorData {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeErrorData {
    pub fn new() -> Self {
        Self {
            next_offset: TOTAL_RESERVED_MEMORY,
            errors: HashMap::new(),
        }
    }

    /// Get the current next offset (the offset where the next error would be written).
    pub fn get_next_offset(&self) -> i32 {
        self.next_offset
    }

    // Returns the offset in the data segment where the ABI-encoded error message for the RuntimeError is stored.
    pub fn get(&mut self, module: &mut Module, memory_id: MemoryId, error: RuntimeError) -> i32 {
        if let Some(&offset) = self.errors.get(&error) {
            return offset;
        }

        let encoded_error = Self::abi_encode_error_message(error);
        let offset = self.next_offset;
        let data_size = encoded_error.len() as i32;

        module.data.add(
            DataKind::Active {
                memory: memory_id,
                offset: ConstExpr::Value(Value::I32(offset)),
            },
            encoded_error,
        );

        self.errors.insert(error, offset);
        self.next_offset += data_size;

        offset
    }

    /// Compute ABI-encoded error bytes at compile time.
    ///
    /// This matches the format produced by `build_error_string_abi_encoding`:
    /// - Bytes 0-3: Total message length (little-endian u32)
    /// - Bytes 4-7: Error selector (4 bytes)
    /// - Bytes 8-39: Head word (32 bytes, with 0x20 at offset 39)
    /// - Bytes 40-71: Length word (32 bytes, big-endian message length at offset 68)
    /// - Bytes 72+: Error message text (padded to 32-byte boundary)
    fn abi_encode_error_message(error: RuntimeError) -> Vec<u8> {
        const LENGTH_HEADER_SIZE: usize = 4;
        const ABI_HEADER_SIZE: usize = 4 + 32 + 32; // selector(4) + head(32) + length(32)

        let error_message = error.to_string();
        let error_bytes = error_message.as_bytes();
        let error_len = error_bytes.len();

        // Round up message length to 32-byte boundary for ABI alignment
        let padded_error_len = (error_len + 31) & !31;

        // Total length of the ABI-encoded message (excluding the length header)
        let total_len = ABI_HEADER_SIZE + padded_error_len;

        let mut result = Vec::with_capacity(LENGTH_HEADER_SIZE + total_len);

        // Write total length (little-endian u32)
        result.extend_from_slice(&(total_len as u32).to_le_bytes());

        // Write error selector (4 bytes)
        result.extend_from_slice(&ERROR_SELECTOR);

        // Write head word (32 bytes, with 0x20 at the last byte)
        let mut head_word = vec![0u8; 32];
        head_word[31] = 0x20;
        result.extend_from_slice(&head_word);

        // Write message length in big-endian format (32 bytes, with length at offset 28-31)
        let mut length_word = vec![0u8; 32];
        let length_be = (error_len as u32).to_be_bytes();
        length_word[28..32].copy_from_slice(&length_be);
        result.extend_from_slice(&length_word);

        // Write error message text
        result.extend_from_slice(error_bytes);

        // Pad to 32-byte boundary
        let padding = padded_error_len - error_len;
        result.extend(vec![0u8; padding]);

        result
    }
}
