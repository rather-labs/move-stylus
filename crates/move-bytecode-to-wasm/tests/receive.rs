mod common;

use common::{runtime_sandbox::RuntimeSandbox, translate_test_package_with_framework};
use rstest::{fixture, rstest};

mod receive {
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "receive";
        const SOURCE_PATH: &str = "tests/receive/receive.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function receive() external payable;
    );

    #[rstest]
    fn test_receive(runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = receiveCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}


mod receive_with_tx_context {
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "receive_with_tx_context";
        const SOURCE_PATH: &str = "tests/receive/receive_with_tx_context.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function receive() external payable;
    );

    #[rstest]
    fn test_receive_with_tx_context(runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = receiveCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}

mod receive_bad {
    use crate::common::translate_test_package_with_framework;
    use rstest::rstest;

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
