mod common;

use crate::common::runtime_with_framework as runtime;
use alloy_sol_types::{SolCall, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

mod receive {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function receive() external payable;
    );

    #[rstest]
    fn test_receive(#[with("receive", "tests/receive/receive.move")] runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = receiveCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}

mod receive_with_tx_context {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function receive() external payable;
    );

    #[rstest]
    fn test_receive_with_tx_context(
        #[with(
            "receive_with_tx_context",
            "tests/receive/receive_with_tx_context.move"
        )]
        runtime: RuntimeSandbox,
    ) {
        // Create a new counter
        let call_data = receiveCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}

mod receive_bad {
    use super::*;
    use crate::common::translate_test_package_with_framework;

    #[rstest]
    #[should_panic(expected = "ReceiveFunctionBadVisibility")]
    #[case("receive_bad_visibility", "tests/receive/receive_bad_visibility.move")]
    #[should_panic(expected = "ReceiveFunctionHasReturns")]
    #[case("receive_bad_returns", "tests/receive/receive_bad_returns.move")]
    #[should_panic(expected = "ReceiveFunctionTooManyArguments")]
    #[case("receive_bad_args_1", "tests/receive/receive_bad_args_1.move")]
    #[should_panic(expected = "ReceiveFunctionNonTxContextArgument")]
    #[case("receive_bad_args_2", "tests/receive/receive_bad_args_2.move")]
    #[should_panic(expected = "ReceiveFunctionIsNotPayable")]
    #[case("receive_bad_mutability", "tests/receive/receive_bad_mutability.move")]
    fn test_receive_bad(#[case] module_name: &str, #[case] source_path: &str) {
        translate_test_package_with_framework(source_path, module_name);
    }
}
