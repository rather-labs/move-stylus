use super::*;
use crate::common::runtime;
use alloy_primitives::{FixedBytes, U256, address, hex, keccak256};
use alloy_sol_types::SolCall;
use alloy_sol_types::sol;
use move_test_runner::constants::MSG_SENDER_ADDRESS;
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
    let object_id = runtime.obtain_uid();

    // Set value to 111 with a sender that is not the owner
    let call_data = adminCapFnCall::new((object_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    // Change the tx origin to change where the contract will look fot the owner
    runtime.set_tx_origin(address!("0x0000000000000000000000000000000abcabcabc").0.0);

    // This call should fails as it did not find the admin
    let call_data = adminCapFnCall::new((object_id,)).abi_encode();
    let result = runtime.call_entrypoint(call_data);
    assert!(result.is_err());
}
