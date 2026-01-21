use crate::common::runtime;
use alloy_primitives::address;
use alloy_sol_types::{SolCall, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::constants::SIGNER_ADDRESS;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]
    function create() public view;
    function adminCapFn(bytes32 id) public view;
);

#[rstest]
fn test_capability(
    #[with("capability", "tests/storage/move_sources/capability.move")] runtime: RuntimeSandbox,
) {
    // Set the sender as the signer, because the owner will be the sender (and we are sending
    // the transaction from the same address that signs it)
    runtime.set_msg_sender(SIGNER_ADDRESS);

    // Create a new counter
    let call_data = createCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Read the object id emmited from the contract's events
    let object_id = runtime.obtain_uid().unwrap();

    // Set value to 111 with a sender that is not the owner
    let call_data = adminCapFnCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Change the tx origin to change where the contract will look fot the owner
    runtime.set_tx_origin(address!("0x0000000000000000000000000000000abcabcabc").0.0);

    // This call should fails as it did not find the admin
    let call_data = adminCapFnCall::new((object_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let expected_data = RuntimeError::StorageObjectNotFound.encode_abi();
    assert_eq!(1, result);
    assert_eq!(expected_data, return_data);
}
