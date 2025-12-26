use crate::common::runtime;
use alloy_sol_types::{SolCall, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

sol!(
    #[allow(missing_docs)]
    function receive() external payable;
);

#[rstest]
fn test_receive(
    #[with("receive", "tests/receive/move_sources/receive.move")] runtime: RuntimeSandbox,
) {
    // Create a new counter
    let call_data = receiveCall::new(()).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
}
