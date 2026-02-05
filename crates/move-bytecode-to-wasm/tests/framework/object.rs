use crate::common::runtime;
use alloy_primitives::address;
use alloy_sol_types::SolCall;
use alloy_sol_types::sol;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[derive(Debug)]
    struct Foo {
        bytes32 id;
        uint64 value;
    }

    #[allow(missing_docs)]
    function createFrozenFoo() external;
    function createSharedFoo() external;
    function createOwnedFoo() external;
    function getFooId(bytes32 id) external returns (bytes32);
    function getFooIdRef(bytes32 id) external returns (bytes32);
    function getFooIdAddress(bytes32 id) external returns (bytes32);
);

#[rstest]
fn test_object_borrow_id(
    #[with("object", "tests/framework/move_sources/object.move")] runtime: RuntimeSandbox,
) {
    // Test owned object
    runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
    runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);
    let call_data = createOwnedFooCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid().unwrap();

    let call_data = getFooIdCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooIdCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(object_id, return_data);
    assert_eq!(0, result);

    let call_data = getFooIdRefCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooIdRefCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(object_id, return_data);
    assert_eq!(0, result);

    let call_data = getFooIdAddressCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooIdAddressCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(object_id, return_data);
    assert_eq!(0, result);

    // Test shared object

    let call_data = createSharedFooCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid().unwrap();

    let call_data = getFooIdCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooIdCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(object_id, return_data);
    assert_eq!(0, result);

    let call_data = getFooIdRefCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooIdRefCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(object_id, return_data);
    assert_eq!(0, result);

    let call_data = getFooIdAddressCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooIdAddressCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(object_id, return_data);
    assert_eq!(0, result);

    // Test frozen object

    let call_data = createFrozenFooCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid().unwrap();

    let call_data = getFooIdCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooIdCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(object_id, return_data);
    assert_eq!(0, result);

    let call_data = getFooIdRefCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooIdRefCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(object_id, return_data);
    assert_eq!(0, result);

    let call_data = getFooIdAddressCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooIdAddressCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(object_id, return_data);
    assert_eq!(0, result);
}
