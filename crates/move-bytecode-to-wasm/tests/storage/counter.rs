use crate::common::runtime;
use alloy_primitives::address;
use alloy_sol_types::SolCall;
use alloy_sol_types::sol;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]
    function create() public view;
    function read(bytes32 id) public view returns (uint64);
    function increment(bytes32 id) public view;
    function setValue(bytes32 id, uint64 value) public view;
);

#[rstest]
fn test_storage_counter(
    #[with("counter", "tests/storage/move_sources/counter.move")] runtime: RuntimeSandbox,
) {
    // Create a new counter
    let call_data = createCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid();

    // Read initial value (should be 25)
    let call_data = readCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(25, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(26, return_data);
    assert_eq!(0, result);

    // Set value to 42
    let call_data = setValueCall::new((object_id, 42)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(42, return_data);
    assert_eq!(0, result);

    // Increment
    let call_data = incrementCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read value
    let call_data = readCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(43, return_data);
    assert_eq!(0, result);

    // change the msg sender
    runtime.set_msg_sender(address!("0x0000000000000000000000000000000abcabcabc").0.0);

    // Set value to 111 with a sender that is not the owner
    let call_data = setValueCall::new((object_id, 111)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(1, result);

    // Assert that the value did not change
    let call_data = readCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = readCall::abi_decode_returns(&return_data).unwrap();
    assert_eq!(43, return_data);
    assert_eq!(0, result);
}
