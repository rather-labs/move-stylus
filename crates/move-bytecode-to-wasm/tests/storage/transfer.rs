use super::*;
use crate::common::runtime;
use alloy_primitives::{FixedBytes, U256, address, hex, keccak256};
use alloy_sol_types::sol;
use alloy_sol_types::{SolCall, SolValue, sol};
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
    struct UID {
       ID id;
    }

    struct Foo {
        UID id;
        uint64 value;
    }

    struct Bar {
        UID id;
        uint64 a;
        uint64[] c;
    }

    struct Qux {
        uint64 a;
        uint128 b;
        uint128 c;
    }

    struct Baz {
        UID id;
        uint64 a;
        Qux c;
    }

    struct Bez {
        UID id;
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
        UID id;
        uint64 a;
        Quz b;
        Quz[] c;
    }

    struct Var {
        UID id;
        Bar a;
    }

    struct Vaz {
        UID id;
        uint32 a;
        Bar b;
        uint64 c;
        Bar d;
    }

    struct EpicVar {
        UID id;
        uint32 a;
        Bar b;
        uint64 c;
        Bar[] d;
    }

    #[allow(missing_docs)]
    function createShared() public view;
    function createOwned(address recipient) public view;
    function createFrozen() public view;
    function readValue(bytes32 id) public view returns (uint64);
    function setValue(bytes32 id, uint64 value) public view;
    function incrementValue(bytes32 id) public view;
    function deleteObj(bytes32 id) public view;
    function freezeObj(bytes32 id) public view;
    function shareObj(bytes32 id) public view;
    function transferObj(bytes32 id, address recipient) public view;
    function getFoo(bytes32 id) public view returns (Foo);
    function createBar() public view;
    function getBar(bytes32 id) public view returns (Bar);
    function deleteBar(bytes32 id) public view;
    function createBaz(address recipient, bool share) public view;
    function getBaz(bytes32 id) public view returns (Baz);
    function deleteBaz(bytes32 id) public view;
    function createBez() public view;
    function getBez(bytes32 id) public view returns (Bez);
    function deleteBez(bytes32 id) public view;
    function createBiz() public view;
    function getBiz(bytes32 id) public view returns (Biz);
    function deleteBiz(bytes32 id) public view;
    function deleteObj2(bytes32 id1, bytes32 id2) public view;
    // Structs with wrapped objects
    function createVar(address recipient) public view;
    function createVarShared() public view;
    function getVar(bytes32 id) public view returns (Var);
    function shareVar(bytes32 id) public view;
    function freezeVar(bytes32 id) public view;
    function deleteVar(bytes32 id) public view;
    function deleteVarAndTransferBar(bytes32 id) public view;
    function createVaz() public view;
    function getVaz(bytes32 id) public view returns (Vaz);
    function deleteVaz(bytes32 id) public view;
    function createEpicVar() public view;
    function getEpicVar(bytes32 id) public view returns (EpicVar);
    function deleteEpicVar(bytes32 id) public view;
);

const SHARED: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
const FROZEN: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
const COUNTER_KEY: [u8; 32] = [
    88, 181, 235, 71, 20, 200, 162, 193, 179, 99, 195, 177, 236, 158, 218, 42, 168, 26, 11, 70, 66,
    173, 6, 207, 222, 175, 248, 56, 236, 49, 87, 253,
];

// Test create frozen object
#[rstest]
fn test_frozen_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createFrozenCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Read value
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);
}

// Tests operations on a shared object: reading, updating values, etc.
#[rstest]
fn test_shared_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    // Create a new counter
    let call_data = createSharedCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Read initial value (should be 101)
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementValueCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(102, return_data);
    assert_eq!(0, result);

    // Set value to 42
    let call_data = setValueCall::new((object_id, 42)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(42, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementValueCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(43, return_data);
    assert_eq!(0, result);

    // Change the signer
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

    // Set value to 111
    let call_data = setValueCall::new((object_id, 111)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Assert that the value is set
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(111, return_data);
    assert_eq!(0, result);

    // Change the signer
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000acacacac").0.0);

    // Increment
    let call_data = incrementValueCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Assert that the value did not change
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(112, return_data);
    assert_eq!(0, result);

    // Change the msg sender
    runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);

    // Set value to 1111 with a sender that is not the owner
    let call_data = setValueCall::new((object_id, 1111)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(1111, return_data);
    assert_eq!(0, result);
}

// Tests operations on an owned object: reading, updating values, etc.
#[rstest]
fn test_owned_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    // Create a new counter
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Read initial value (should be 101)
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementValueCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(102, return_data);
    assert_eq!(0, result);

    // Set value to 42
    let call_data = setValueCall::new((object_id, 42)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(42, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementValueCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(43, return_data);
    assert_eq!(0, result);

    // Change the msg sender
    // Should still work since the signer is the owner
    runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);

    // Set value to 111 with a sender that is not the owner
    let call_data = setValueCall::new((object_id, 111)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Assert that the value was changes correctly
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(111, return_data);
    assert_eq!(0, result);

    // Delete object
    let call_data = deleteObjCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}

// Tests the share of an object in both owned and shared cases.
#[rstest]
fn test_share_owned_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    // Create a new object
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Compute the object slot using the owner and the object id
    let owner = runtime.get_tx_origin();
    let object_slot = derive_object_slot(&owner, &object_id.0);

    // Read the storage on the original slot before the freeze
    let value_before_share = runtime.get_storage_at_slot(object_slot.0);

    // Share the object. Only possible if the object is owned by the signer!
    let call_data = shareObjCall::new((object_id,)).abi_encode();
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
    let shared_slot = derive_object_slot(&SHARED, &object_id.0);

    // Read the storage on the shared slot after the share
    // Should be the same as the original slot before the share
    let shared_value = runtime.get_storage_at_slot(shared_slot.0);
    assert_eq!(
        value_before_share, shared_value,
        "Expected storage value to be the same"
    );

    // Read value
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Change the signer and read again
    // Should still work since the object is shared
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);
}

// Tests the freeze of an object in both owned case.
#[rstest]
fn test_freeze_owned_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    // Create a new object
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Compute the object slot using the owner and the object id
    let owner = runtime.get_tx_origin();
    let object_slot = derive_object_slot(&owner, &object_id.0);

    // Read the storage on the original slot before the freeze
    let value_before_freeze = runtime.get_storage_at_slot(object_slot.0);

    // Freeze the object. Only possible if the object is owned by the signer!
    let call_data = freezeObjCall::new((object_id,)).abi_encode();
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
    let frozen_slot = derive_object_slot(&FROZEN, &object_id.0);

    // Read the storage on the frozen slot after the freeze
    let frozen_value = runtime.get_storage_at_slot(frozen_slot.0);
    assert_eq!(
        value_before_freeze, frozen_value,
        "Expected storage value to be the same"
    );

    // Read value
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Change the signer and read again
    // Should still work since the object is frozen
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Change the msg sender and read again
    // Should still work since the object is frozen
    runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);
}

// Tests trying to read an owned object with a signer that is not the owner.
#[rstest]
#[should_panic(expected = "unreachable")]
fn test_signer_owner_mismatch(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Read initial value (should be 101)
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // change the signer
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

    // This should hit an unreachable due to the signer differing from the owner!
    let call_data = readValueCall::new((object_id,)).abi_encode();
    runtime.call_entrypoint(call_data).unwrap();
}

// Tests the freeze of an object that is not owned by the signer.
#[rstest]
#[should_panic(expected = "unreachable")]
fn test_freeze_not_owned_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

    // Freeze the object. Only possible if the object is owned by the signer!
    let call_data = freezeObjCall::new((object_id,)).abi_encode();
    runtime.call_entrypoint(call_data).unwrap();
}

// Tests the freeze of a shared object.
#[rstest]
#[should_panic(expected = "unreachable")]
fn test_freeze_shared_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    // Create a new object
    let call_data = createSharedCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Freeze the object. Only possible if the object is owned by the signer!
    let call_data = freezeObjCall::new((object_id,)).abi_encode();
    runtime.call_entrypoint(call_data).unwrap();
}

// Freeze and then try to share or transfer the object.
#[rstest]
#[should_panic(expected = "unreachable")]
#[case(false)]
#[should_panic(expected = "unreachable")]
#[case(true)]
fn test_share_or_transfer_frozen(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
    #[case] share: bool,
) {
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Freeze the object. Only possible if the object is owned by the signer!
    let call_data = freezeObjCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    if share {
        // Try to share the object.
        let call_data = shareObjCall::new((object_id,)).abi_encode();
        runtime.call_entrypoint(call_data).unwrap();
    } else {
        // Try to transfer the object.
        let call_data = transferObjCall::new((object_id, SIGNER_ADDRESS.into())).abi_encode();
        runtime.call_entrypoint(call_data).unwrap();
    }
}

#[rstest]
#[should_panic(expected = "unreachable")]
fn test_delete_frozen_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createFrozenCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = runtime.obtain_uid();

    // Read value before delete
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    // Try to delete the object
    let call_data = deleteObjCall::new((object_id,)).abi_encode();
    runtime.call_entrypoint(call_data).unwrap();
}

// Test delete owned object
#[rstest]
fn test_delete_owned_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = runtime.obtain_uid();

    // Read value before delete
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    // Delete the object
    let call_data = deleteObjCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_delete_shared_object(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createSharedCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = runtime.obtain_uid();

    // Read value before delete
    let call_data = readValueCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(101, return_data);
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    // Delete the object
    let call_data = deleteObjCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_get_foo(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    // Create a new counter
    let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Set value to 111 with a sender that is not the owner
    let call_data = getFooCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Foo::abi_encode(&Foo {
        id: UID {
            id: ID { bytes: object_id },
        },
        value: 101,
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);
}

#[rstest]
fn test_delete_bar(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createBarCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    let object_id = runtime.obtain_uid();

    let call_data = getBarCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Bar::abi_encode(&Bar {
        id: UID {
            id: ID { bytes: object_id },
        },
        a: 101,
        c: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let call_data = deleteBarCall::new((object_id,)).abi_encode();
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
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
    #[case] share: bool,
) {
    let call_data = createBazCall::new((SIGNER_ADDRESS.into(), share)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    let object_id = runtime.obtain_uid();

    let call_data = getBazCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Baz::abi_encode(&Baz {
        id: UID {
            id: ID { bytes: object_id },
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

    let call_data = deleteBazCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_delete_bez(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createBezCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    let object_id = runtime.obtain_uid();

    let call_data = getBezCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Bez::abi_encode(&Bez {
        id: UID {
            id: ID { bytes: object_id },
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

    let call_data = deleteBezCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    for (key, value) in storage_after_delete.iter() {
        if *key != COUNTER_KEY {
            // Assert that the key existed in storage before deletion
            assert!(
                storage_before_delete.contains_key(key),
                "Key {key:?} should exist in storage_before_delete"
            );

            assert_eq!(
                *value, [0u8; 32],
                "Unexpected non-zero value at key: {key:?}"
            );
        }
    }
}

#[rstest]
fn test_delete_biz(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createBizCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_before_delete = runtime.get_storage();

    let object_id = runtime.obtain_uid();

    let call_data = getBizCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Biz::abi_encode(&Biz {
        id: UID {
            id: ID { bytes: object_id },
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

    let call_data = deleteBizCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    for (key, value) in storage_after_delete.iter() {
        if *key != COUNTER_KEY {
            // Assert that the key existed in storage before deletion
            assert!(
                storage_before_delete.contains_key(key),
                "Key {key:?} should exist in storage_before_delete"
            );

            assert_eq!(
                *value, [0u8; 32],
                "Unexpected non-zero value at key: {key:?}"
            );
        }
    }
}

#[rstest]
fn test_delete_many(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createSharedCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_1_id = runtime.obtain_uid();

    let call_data = createSharedCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_2_id = runtime.obtain_uid();

    let storage_before_delete = runtime.get_storage();

    let call_data = deleteObj2Call::new((object_1_id, object_2_id)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_delete_owned_var(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createVarCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = runtime.obtain_uid();

    let call_data = getVarCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Var::abi_encode(&Var {
        id: UID {
            id: ID {
                bytes: U256::from_str_radix(
                    "facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb",
                    16,
                )
                .unwrap()
                .into(),
            },
        },
        a: Bar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 42,
            c: vec![1, 2, 3],
        },
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let storage_before_delete = runtime.get_storage();

    let call_data = deleteVarCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

// First we test deleting a Var shared upon creation
// Then we test deleting a Var owned upon creation and later shared
#[rstest]
fn test_delete_shared_var(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createVarSharedCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = runtime.obtain_uid();

    let call_data = getVarCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Var::abi_encode(&Var {
        id: UID {
            id: ID {
                bytes: U256::from_str_radix(
                    "facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb",
                    16,
                )
                .unwrap()
                .into(),
            },
        },
        a: Bar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 42,
            c: vec![1, 2, 3],
        },
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let storage_before_delete = runtime.get_storage();

    let call_data = deleteVarCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);

    // Create owned var and share it, then delete it
    let call_data = createVarCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = FixedBytes::<32>::from_slice(
        &hex::decode("79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09").unwrap(),
    );

    let call_data = getVarCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Var::abi_encode(&Var {
        id: UID {
            id: ID {
                bytes: U256::from_str_radix(
                    "79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09",
                    16,
                )
                .unwrap()
                .into(),
            },
        },
        a: Bar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "12b23b08610619d2c73d9c594768afa7bcc248bd34e1f202173e5c92014ae02e",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 42,
            c: vec![1, 2, 3],
        },
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let call_data = shareVarCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = getVarCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Var::abi_encode(&Var {
        id: UID {
            id: ID {
                bytes: U256::from_str_radix(
                    "79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09",
                    16,
                )
                .unwrap()
                .into(),
            },
        },
        a: Bar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "12b23b08610619d2c73d9c594768afa7bcc248bd34e1f202173e5c92014ae02e",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 42,
            c: vec![1, 2, 3],
        },
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let storage_before_delete = runtime.get_storage();

    let call_data = deleteVarCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_freeze_owned_var(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    // Create owned var and freeze it
    let call_data = createVarCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = FixedBytes::<32>::from_slice(
        &hex::decode("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb").unwrap(),
    );

    let call_data = freezeVarCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Check if the 2 slots corresponding to the Var struct under the SIGNER_ADDRESS key are empty after the freeze
    let var_uid_slot_bytes: [u8; 32] = [
        68, 3, 231, 226, 205, 228, 8, 70, 51, 84, 182, 15, 113, 190, 199, 118, 176, 64, 3, 212,
        161, 124, 104, 159, 179, 185, 36, 30, 225, 140, 146, 77,
    ];
    let bar_uid_slot_bytes: [u8; 32] = [
        68, 3, 231, 226, 205, 228, 8, 70, 51, 84, 182, 15, 113, 190, 199, 118, 176, 64, 3, 212,
        161, 124, 104, 159, 179, 185, 36, 30, 225, 140, 146, 78,
    ];

    // Check if the slots exist and are zero
    let value = runtime.get_storage_at_slot(var_uid_slot_bytes);
    assert_eq!(value, [0u8; 32], "Var UID slot should be zero");

    let value = runtime.get_storage_at_slot(bar_uid_slot_bytes);
    assert_eq!(value, [0u8; 32], "Bar UID slot should be zero");

    let call_data = getVarCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Var::abi_encode(&Var {
        id: UID {
            id: ID {
                bytes: U256::from_str_radix(
                    "facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb",
                    16,
                )
                .unwrap()
                .into(),
            },
        },
        a: Bar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 42,
            c: vec![1, 2, 3],
        },
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);
}

#[rstest]
// Delete var and share bar
// Check that all original slots are empty: Var is delete and Bar is moved
// After that, try getting bar and then deleting it
fn test_delete_var_and_transfer_bar(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createVarCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = FixedBytes::<32>::from_slice(
        &hex::decode("facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb").unwrap(),
    );

    let storage_before_delete = runtime.get_storage();

    // Delete var and share bar
    let call_data = deleteVarAndTransferBarCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete: std::collections::HashMap<[u8; 32], [u8; 32]> = runtime.get_storage();
    for key in storage_before_delete.keys() {
        if *key != COUNTER_KEY {
            let value = storage_after_delete.get(key).unwrap_or(&[0u8; 32]);
            assert_eq!(
                *value, [0u8; 32],
                "Unexpected non-zero value at key: {key:?}"
            );
        }
    }

    // Bar id
    let object_id = FixedBytes::<32>::from_slice(
        &hex::decode("e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7").unwrap(),
    );

    let call_data = getBarCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    let expected_result = Bar::abi_encode(&Bar {
        id: UID {
            id: ID {
                bytes: U256::from_str_radix(
                    "e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7",
                    16,
                )
                .unwrap()
                .into(),
            },
        },
        a: 42,
        c: vec![1, 2, 3],
    });
    assert_eq!(result_data, expected_result);

    let storage_before_delete = runtime.get_storage();

    let call_data = deleteBarCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_delete_vaz(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createVazCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = runtime.obtain_uid();

    let call_data = getVazCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_result = Vaz::abi_encode(&Vaz {
        id: UID {
            id: ID {
                bytes: U256::from_str_radix(
                    "facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb",
                    16,
                )
                .unwrap()
                .into(),
            },
        },
        a: 101,
        b: Bar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 42,
            c: vec![1, 2, 3],
        },
        c: 102,
        d: Bar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 43,
            c: vec![4, 5, 6],
        },
    });
    assert_eq!(0, result);
    assert_eq!(result_data, expected_result);

    let storage_before_delete = runtime.get_storage();

    let call_data = deleteVazCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}

#[rstest]
fn test_delete_epic_var(
    #[with("transfer", "tests/storage/move_sources/transfer.move")] runtime: RuntimeSandbox,
) {
    let call_data = createEpicVarCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let object_id = runtime.obtain_uid();

    let call_data = getEpicVarCall::new((object_id,)).abi_encode();
    let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    let expected_result = EpicVar::abi_encode(&EpicVar {
        id: UID {
            id: ID {
                bytes: U256::from_str_radix(
                    "facda8b03f21c31df6f060ec021902355a60f784caacfca695acb879d66e76cb",
                    16,
                )
                .unwrap()
                .into(),
            },
        },
        a: 101,
        b: Bar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "e014f8017b7a8c4a930b9b7fcf7731e1a3d955813e4d729c5abf81df5adb08a7",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 41,
            c: vec![1, 2, 3],
        },
        c: 102,
        d: vec![
            Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "79f6f905732424817cc3297d425cd1313a7afd112df46d08303219989d6a7b09",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 42,
                c: vec![42, 43],
            },
            Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "12b23b08610619d2c73d9c594768afa7bcc248bd34e1f202173e5c92014ae02e",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 43,
                c: vec![44, 45, 46],
            },
        ],
    });
    assert_eq!(result_data, expected_result);

    let storage_before_delete = runtime.get_storage();

    let call_data = deleteEpicVarCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let storage_after_delete = runtime.get_storage();

    // Assert that all storage slots are empty except for the specified key
    assert_empty_storage(&storage_before_delete, &storage_after_delete);
}
