use super::*;
use crate::common::runtime;
use alloy_primitives::{FixedBytes, U256, address, hex, keccak256};
use alloy_sol_types::sol;
use alloy_sol_types::{SolCall, SolValue};
use move_test_runner::constants::MSG_SENDER_ADDRESS;
use move_test_runner::constants::SIGNER_ADDRESS;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]

    #[derive(Debug)]
    struct ID {
       bytes32 bytes;
    }

    #[derive(Debug)]
    struct NamedId {
       ID id;
    }

    struct Foo {
        NamedId id;
        uint64 value;
    }

    struct Bar {
        NamedId id;
        uint64 a;
        uint64[] c;
    }

    struct Qux {
        uint64 a;
        uint128 b;
        uint128 c;
    }

    struct Baz {
        NamedId id;
        uint64 a;
        Qux c;
    }

    struct Bez {
        NamedId id;
        uint64 a;
        Qux[] c;
        uint128[][] d;
        uint8 e;
    }

    struct Quz {
        uint64 a;
        uint128 b;
        uint128 c;
    }

    struct Biz {
        NamedId id;
        uint64 a;
        Quz b;
        Quz[] c;
    }


    #[allow(missing_docs)]
    function createShared() public view;
    function createOwned(address recipient) public view;
    function createFrozen() public view;
    function readValue() public view returns (uint64);
    function setValue(uint64 value) public view;
    function incrementValue() public view;
    function deleteObj() public view;
    function freezeObj() public view;
    function shareObj() public view;
    function transferObj(address recipient) public view;
    function getFoo() public view returns (Foo);
    function createBar() public view;
    function getBar() public view returns (Bar);
    function deleteBar() public view;
    function createBaz(address recipient, bool share) public view;
    function getBaz() public view returns (Baz);
    function deleteBaz() public view;
    function createBez() public view;
    function getBez() public view returns (Bez);
    function deleteBez() public view;
    function createBiz() public view;
    function getBiz() public view returns (Biz);
    function deleteBiz() public view;
);

const FOO_ID: [u8; 32] = [
    0x04, 0xd9, 0x56, 0x9c, 0xa9, 0x35, 0xbc, 0x8c, 0x4d, 0x83, 0x5e, 0xf7, 0x49, 0xba, 0x26, 0x04,
    0x0d, 0x5a, 0x5a, 0xbc, 0x33, 0xe3, 0xe3, 0x4e, 0x4a, 0x9e, 0xe6, 0x91, 0xe2, 0x93, 0x4f, 0xc7,
];
const BAR_ID: [u8; 32] = [
    0x07, 0xcc, 0xac, 0x83, 0x2b, 0x4b, 0x6e, 0x44, 0x05, 0x80, 0xae, 0x89, 0x26, 0xba, 0xcf, 0x74,
    0xae, 0xe4, 0xf1, 0x90, 0x78, 0x2a, 0x69, 0xed, 0x94, 0x80, 0xee, 0x90, 0xd2, 0xac, 0x90, 0x87,
];
const BAZ_ID: [u8; 32] = [
    0xcc, 0xd6, 0xf0, 0x70, 0x9a, 0xda, 0xce, 0xfb, 0xfc, 0x3b, 0x75, 0x15, 0x74, 0x62, 0x0a, 0xf6,
    0x39, 0xb0, 0x09, 0x14, 0x44, 0x16, 0x68, 0x40, 0xd7, 0x02, 0x9b, 0x05, 0x10, 0x9d, 0x69, 0xa0,
];
const BEZ_ID: [u8; 32] = [
    0xdb, 0xb5, 0x11, 0xf8, 0x0e, 0x54, 0xba, 0xb0, 0x5d, 0x9e, 0xa1, 0xbf, 0x80, 0xca, 0xfc, 0x3e,
    0x73, 0xd1, 0x6f, 0x00, 0x09, 0xd7, 0x19, 0xd4, 0x1b, 0x78, 0xb1, 0xc5, 0x0d, 0x3b, 0x4c, 0x82,
];
const BIZ_ID: [u8; 32] = [
    0x4a, 0x4f, 0xb7, 0xff, 0x47, 0x1d, 0xd5, 0xee, 0xc1, 0x2e, 0xde, 0x27, 0x7e, 0xea, 0x16, 0xdc,
    0x35, 0x8f, 0xef, 0x5a, 0x22, 0x54, 0x83, 0xfd, 0xee, 0x94, 0x3d, 0x54, 0xf0, 0x75, 0xc0, 0xc8,
];

// Test create frozen object
#[rstest]
fn test_frozen_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = createFrozenCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);
}

// Tests operations on a shared object: reading, updating values, etc.
#[rstest]
fn test_shared_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    // Create a new counter
    let call_data = createSharedCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read initial value (should be 101)
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementValueCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(102, return_data);
    assert_eq!(0, result);

    // Set value to 42
    let call_data = setValueCall::new((42,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(42, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementValueCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(43, return_data);
    assert_eq!(0, result);

    // Change the signer
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

    // Set value to 111
    let call_data = setValueCall::new((111,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Assert that the value is set
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(111, return_data);
    assert_eq!(0, result);

    // Change the signer
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000acacacac").0.0);

    // Increment
    let call_data = incrementValueCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Assert that the value did not change
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(112, return_data);
    assert_eq!(0, result);

    // Change the msg sender
    runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);

    // Set value to 1111 with a sender that is not the owner
    let call_data = setValueCall::new((1111,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(1111, return_data);
    assert_eq!(0, result);
}

// Tests operations on an owned object: reading, updating values, etc.
#[rstest]
fn test_owned_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    // Create a new counter
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read initial value (should be 101)
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementValueCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(102, return_data);
    assert_eq!(0, result);

    // Set value to 42
    let call_data = setValueCall::new((42,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(42, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementValueCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(43, return_data);
    assert_eq!(0, result);

    // Change the msg sender
    // Should still work since the signer is the owner
    runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);

    // Set value to 111 with a sender that is not the owner
    let call_data = setValueCall::new((111,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Assert that the value was changes correctly
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(111, return_data);
    assert_eq!(0, result);

    // Delete object
    let call_data = deleteObjCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}

// Tests the share of an object in both owned and shared cases.
#[rstest]
fn test_share_owned_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    // Create a new object
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Compute the object slot using the owner and the object id
    let owner = runtime.get_tx_origin();
    let object_slot = derive_object_slot(&owner, &FOO_ID);

    // Read the storage on the original slot before the freeze
    let value_before_share = runtime.get_storage_at_slot(object_slot.0);

    // Share the object. Only possible if the object is owned by the signer!
    let call_data = shareObjCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the storage on the original slot after the share
    // Should be zeroes since the object moved from the owner space to the shared space
    let value_after_share = runtime.get_storage_at_slot(object_slot.0);
    assert_eq!(
        [0u8; 32], value_after_share,
        "Expected storage value to be 32 zeros"
    );

    // Get the slot number for the shared object
    let shared_slot = derive_object_slot(&SHARED, &FOO_ID);

    // Read the storage on the shared slot after the share
    // Should be the same as the original slot before the share
    let shared_value = runtime.get_storage_at_slot(shared_slot.0);
    assert_eq!(
        value_before_share, shared_value,
        "Expected storage value to be the same"
    );

    // Read value
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Change the signer and read again
    // Should still work since the object is shared
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);
}

// Tests the freeze of an object in both owned case.
#[rstest]
fn test_freeze_owned_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    // Create a new object
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Compute the object slot using the owner and the object id
    let owner = runtime.get_tx_origin();
    let object_slot = derive_object_slot(&owner, &FOO_ID);

    // Read the storage on the original slot before the freeze
    let value_before_freeze = runtime.get_storage_at_slot(object_slot.0);

    // Freeze the object. Only possible if the object is owned by the signer!
    let call_data = freezeObjCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the storage on the original slot after the freeze
    // Should be zeroes since the object moved from the owner space to the frozen space
    let value_after_freeze = runtime.get_storage_at_slot(object_slot.0);
    assert_eq!(
        [0u8; 32], value_after_freeze,
        "Expected storage value to be 32 zeros"
    );

    // Compute the object slot using the FROZEN address and the object id
    let frozen_slot = derive_object_slot(&FROZEN, &FOO_ID);

    // Read the storage on the frozen slot after the freeze
    let frozen_value = runtime.get_storage_at_slot(frozen_slot.0);
    assert_eq!(
        value_before_freeze, frozen_value,
        "Expected storage value to be the same"
    );

    // Read value
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Change the signer and read again
    // Should still work since the object is frozen
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Change the msg sender and read again
    // Should still work since the object is frozen
    runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);
}

// Tests trying to read an owned object with a signer that is not the owner.
#[rstest]
#[should_panic(expected = "unreachable")]
fn test_signer_owner_mismatch(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read initial value (should be 101)
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // change the signer
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

    // This should hit an unreachable due to the signer differing from the owner!
    let call_data = readValueCall::new(()).abi_encode();
    runtime.call_entrypoint(call_data).unwrap();
}

// Tests the freeze of an object that is not owned by the signer.
#[rstest]
#[should_panic(expected = "unreachable")]
fn test_freeze_not_owned_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

    // Freeze the object. Only possible if the object is owned by the signer!
    let call_data = freezeObjCall::new(()).abi_encode();
    runtime.call_entrypoint(call_data).unwrap();
}

// Tests the freeze of a shared object.
#[rstest]
#[should_panic(expected = "unreachable")]
fn test_freeze_shared_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    // Create a new object
    let call_data = createSharedCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Freeze the object. Only possible if the object is owned by the signer!
    let call_data = freezeObjCall::new(()).abi_encode();
    runtime.call_entrypoint(call_data).unwrap();
}

// Freeze and then try to share or transfer the object.
#[rstest]
#[should_panic(expected = "unreachable")]
#[case(false)]
#[should_panic(expected = "unreachable")]
#[case(true)]
fn test_share_or_transfer_frozen(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
    #[case] share: bool,
) {
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Freeze the object. Only possible if the object is owned by the signer!
    let call_data = freezeObjCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    if share {
        // Try to share the object.
        let call_data = shareObjCall::new(()).abi_encode();
        runtime.call_entrypoint(call_data).unwrap();
    } else {
        // Try to transfer the object.
        let call_data = transferObjCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        runtime.call_entrypoint(call_data).unwrap();
    }
}

#[rstest]
#[should_panic(expected = "unreachable")]
fn test_delete_frozen_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = createFrozenCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value before delete
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Try to delete the object
    let call_data = deleteObjCall::new(()).abi_encode();
    runtime.call_entrypoint(call_data).unwrap();
}

// Test delete owned object
#[rstest]
fn test_delete_owned_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    // Read value before delete
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Delete the object
    let call_data = deleteObjCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

// Test delete owned object
#[rstest]
fn test_delete_shared_object(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = createSharedCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    // Read value before delete
    let call_data = readValueCall::new(()).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Delete the object
    let call_data = deleteObjCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_get_foo(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    // Create a new counter
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Set value to 111 with a sender that is not the owner
    let call_data = getFooCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Foo::abi_encode(&Foo {
        id: NamedId {
            id: ID {
                bytes: alloy_primitives::FixedBytes(FOO_ID),
            },
        },
        value: 101,
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);
}

#[rstest]
fn test_delete_bar(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = createBarCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    let call_data = getBarCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Bar::abi_encode(&Bar {
        id: NamedId {
            id: ID {
                bytes: alloy_primitives::FixedBytes(BAR_ID),
            },
        },
        a: 101,
        c: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let call_data = deleteBarCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
#[case(false)]
#[case(true)]
fn test_delete_baz(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
    #[case] share: bool,
) {
    let call_data = createBazCall::new((SIGNER_ADDRESS.into(), share)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    let call_data = getBazCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Baz::abi_encode(&Baz {
        id: NamedId {
            id: ID {
                bytes: alloy_primitives::FixedBytes(BAZ_ID),
            },
        },
        a: 101,
        c: Qux {
            a: 42,
            b: 55,
            c: 66,
        },
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let call_data = deleteBazCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_delete_bez(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = createBezCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    let call_data = getBezCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Bez::abi_encode(&Bez {
        id: NamedId {
            id: ID {
                bytes: alloy_primitives::FixedBytes(BEZ_ID),
            },
        },
        a: 101,
        c: vec![
            Qux {
                a: 42,
                b: 55,
                c: 66,
            },
            Qux {
                a: 43,
                b: 56,
                c: 67,
            },
            Qux {
                a: 44,
                b: 57,
                c: 68,
            },
        ],
        d: vec![vec![1, 2, 3], vec![4], vec![], vec![5, 6]],
        e: 17,
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let call_data = deleteBezCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_delete_biz(
    #[with(
        "transfer_named_id",
        "tests/storage/move_sources/transfer_named_id.move"
    )]
    runtime: RuntimeSandbox,
) {
    let call_data = createBizCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    let call_data = getBizCall::new(()).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Biz::abi_encode(&Biz {
        id: NamedId {
            id: ID {
                bytes: alloy_primitives::FixedBytes(BIZ_ID),
            },
        },
        a: 101,
        b: Quz {
            a: 42,
            b: 55,
            c: 66,
        },
        c: vec![
            Quz {
                a: 42,
                b: 55,
                c: 66,
            },
            Quz {
                a: 43,
                b: 56,
                c: 67,
            },
            Quz {
                a: 44,
                b: 57,
                c: 68,
            },
        ],
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let call_data = deleteBizCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}
