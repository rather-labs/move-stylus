mod capability;
mod counter;
mod counter_named_id;
mod dynamic_fields;
mod dynamic_fields_named_id;
mod dynamic_table;
mod encoding;
mod enums;
mod erc20;
mod simple_warrior;
mod storage_modifiers;
mod transfer;
mod transfer_named_id;
mod trusted_swap;
mod wrapped_objects;

use alloy_primitives::{FixedBytes, U256, keccak256};

pub const SHARED: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
pub const FROZEN: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
pub const COUNTER_KEY: [u8; 32] = [
    88, 181, 235, 71, 20, 200, 162, 193, 179, 99, 195, 177, 236, 158, 218, 42, 168, 26, 11, 70, 66,
    173, 6, 207, 222, 175, 248, 56, 236, 49, 87, 253,
];

/// Right-align `data` into a 32-byte word (EVM storage encoding for value types).
#[inline]
fn pad32_right(data: &[u8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    let n = data.len().min(32);
    out[32 - n..].copy_from_slice(&data[..n]); // <-- right-align
    out
}

/// mapping(address => mapping(bytes32 => V)) at base slot 0
/// slot(owner, id) = keccak256( pad32(id) || keccak256( pad32(owner) || pad32(0) ) )
pub fn derive_object_slot(owner: &[u8], object_id: &[u8]) -> FixedBytes<32> {
    // parent = keccak256( pad32(owner) || pad32(0) )
    let owner_padded = pad32_right(owner);
    let zero_slot = [0u8; 32];

    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(&owner_padded);
    buf[32..].copy_from_slice(&zero_slot);
    let parent = keccak256(buf);

    // slot = keccak256( pad32(id) || pad32(parent) )
    let id_padded = pad32_right(object_id); // object_id is already 32B, this is a no-op
    buf[..32].copy_from_slice(&id_padded);
    buf[32..].copy_from_slice(parent.as_slice());
    keccak256(buf)
}

pub fn get_next_slot(slot: &[u8; 32]) -> [u8; 32] {
    let slot_value = U256::from_be_bytes(*slot);
    (slot_value + U256::from(1)).to_be_bytes()
}

// Helper function to assert that all storage slots are empty after a delete operation
// It checks that keys from before_delete now have zero values in after_delete
pub fn assert_empty_storage(
    storage_before_delete: &std::collections::HashMap<[u8; 32], [u8; 32]>,
    storage_after_delete: &std::collections::HashMap<[u8; 32], [u8; 32]>,
) {
    // Check that keys that existed before delete now have zero values after delete, except for the counter key
    for key in storage_before_delete.keys() {
        if *key != COUNTER_KEY {
            let value_after = storage_after_delete.get(key).unwrap_or(&[0u8; 32]);
            assert_eq!(
                *value_after, [0u8; 32],
                "Unexpected non-zero value at key {key:?} after delete"
            );
        }
    }
}
