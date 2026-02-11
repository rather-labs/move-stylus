use crate::common::runtime;
use alloy_primitives::keccak256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::constants::SIGNER_ADDRESS;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]
    function createOwnedObject() public view;
    function createSharedObject() public view;
    function createFrozenObject() public view;

    function locateOwnedObjectFn(bytes32 id) public view returns (uint64);
    function locateSharedObjectFn(bytes32 id) public view returns (uint64);
    function locateFrozenObjectFn(bytes32 id) public view returns (uint64);
    function locateObjectNoModifierFn(bytes32 id) public view returns (uint64);
    function locateManyObjectsFn(bytes32 a, bytes32 b, bytes32 c) public view returns (uint64);
    function locateManyObjects2Fn(bytes32 a, bytes32 b, bytes32 c) public view returns (uint64);
);

#[rstest]
fn test_storage_modifiers(
    #[with(
        "storage_modifiers",
        "tests/storage/move_sources/storage_modifiers.move"
    )]
    runtime: RuntimeSandbox,
) {
    // Set the sender as the signer, because the owner will be the sender (and we are sending
    // the transaction from the same address that signs it)
    runtime.set_msg_sender(SIGNER_ADDRESS);

    // Create an owned object
    let call_data = createOwnedObjectCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let owned_object_id = runtime.obtain_uid().unwrap();

    // Create a shared object
    let call_data = createSharedObjectCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let shared_object_id = runtime.obtain_uid().unwrap();

    // Create a frozen object
    let call_data = createFrozenObjectCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let frozen_object_id = runtime.obtain_uid().unwrap();

    // Locate the owned object
    let call_data = locateOwnedObjectFnCall::new((owned_object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = locateOwnedObjectFnCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(42, return_data);
    assert_eq!(0, result);

    // Locate the shared object
    let call_data = locateSharedObjectFnCall::new((shared_object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = locateSharedObjectFnCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(43, return_data);
    assert_eq!(0, result);

    // Locate the frozen object
    let call_data = locateFrozenObjectFnCall::new((frozen_object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = locateFrozenObjectFnCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(44, return_data);
    assert_eq!(0, result);

    // Locate the objects without using storage modifiers -> LocateStorageData path
    let call_data = locateObjectNoModifierFnCall::new((owned_object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = locateObjectNoModifierFnCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(42, return_data);
    assert_eq!(0, result);

    let call_data = locateObjectNoModifierFnCall::new((shared_object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = locateObjectNoModifierFnCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(43, return_data);
    assert_eq!(0, result);

    let call_data = locateObjectNoModifierFnCall::new((frozen_object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = locateObjectNoModifierFnCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(44, return_data);
    assert_eq!(0, result);

    // Locate many objects: each argument has a different storage modifier
    let call_data =
        locateManyObjectsFnCall::new((owned_object_id, shared_object_id, frozen_object_id))
            .abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = locateManyObjectsFnCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(129, return_data);
    assert_eq!(0, result);

    // Locate many objects: storage modifier for owned object is not set, hence LocateStorageData is used to load that one
    let call_data =
        locateManyObjects2FnCall::new((owned_object_id, shared_object_id, frozen_object_id))
            .abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = locateManyObjects2FnCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(129, return_data);
    assert_eq!(0, result);

    // Locate the object with a bad modifier — calling locateOwnedObjectFn with a shared object id
    // should fail because the object won't be found in the owned storage mapping.
    let call_data = locateOwnedObjectFnCall::new((shared_object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(1, result);

    let object_not_found_error = [
        keccak256(b"Error(string)")[..4].to_vec(),
        <sol!((string,))>::abi_encode_params(&("Object not found",)),
    ]
    .concat();
    assert_eq!(return_data, object_not_found_error);

    // Trying to locate a frozen object using shared modifier
    let call_data = locateSharedObjectFnCall::new((frozen_object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(1, result);
    assert_eq!(return_data, object_not_found_error);

    // Here we swap frozen and owned objects so this test should fail
    let call_data =
        locateManyObjectsFnCall::new((frozen_object_id, shared_object_id, owned_object_id))
            .abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(return_data, object_not_found_error);
    assert_eq!(1, result);
}
