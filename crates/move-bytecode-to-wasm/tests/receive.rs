mod common;

use common::{runtime_sandbox::RuntimeSandbox, translate_test_package_with_framework_result};
use rstest::{fixture, rstest};

mod receive {
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "receive";
        const SOURCE_PATH: &str = "tests/receive/receive.move";

        let mut translated_package =
            translate_test_package_with_framework_result(SOURCE_PATH, MODULE_NAME).unwrap();

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

mod receive_bad {
    use crate::common::translate_test_package_with_framework_result;
    use move_bytecode_to_wasm::compilation_context::CompilationContextError;
    use move_bytecode_to_wasm::error::{CompilationError, ICEErrorKind};

    #[test]
    fn test_receive_bad_visibility() {
        const MODULE_NAME: &str = "receive_bad_visibility";
        const SOURCE_PATH: &str = "tests/receive/receive_bad_visibility.move";

        let result = translate_test_package_with_framework_result(SOURCE_PATH, MODULE_NAME);

        match result {
            Ok(_) => panic!("Expected translation to fail with ReceiveFunctionBadVisibility error"),
            Err(CompilationError::ICE(ice_error)) => {
                match ice_error.kind {
                    ICEErrorKind::CompilationContext(
                        CompilationContextError::ReceiveFunctionBadVisibility,
                    ) => {
                        // Correct error!
                    }
                    other => panic!("Expected ReceiveFunctionBadVisibility, got {other:?}"),
                }
            }
            Err(other) => panic!("Expected ICE error, got {other:?}"),
        }
    }

    #[test]
    fn test_receive_bad_returns() {
        const MODULE_NAME: &str = "receive_bad_returns";
        const SOURCE_PATH: &str = "tests/receive/receive_bad_returns.move";

        let result = translate_test_package_with_framework_result(SOURCE_PATH, MODULE_NAME);

        match result {
            Ok(_) => panic!("Expected translation to fail with ReceiveFunctionHasReturns error"),
            Err(CompilationError::ICE(ice_error)) => {
                match ice_error.kind {
                    ICEErrorKind::CompilationContext(
                        CompilationContextError::ReceiveFunctionHasReturns,
                    ) => {
                        // Correct error!
                    }
                    other => panic!("Expected ReceiveFunctionHasReturns, got {other:?}"),
                }
            }
            Err(other) => panic!("Expected ICE error, got {other:?}"),
        }
    }

    #[test]
    fn test_receive_bad_args() {
        const MODULE_NAME: &str = "receive_bad_args";
        const SOURCE_PATH: &str = "tests/receive/receive_bad_args.move";

        let result = translate_test_package_with_framework_result(SOURCE_PATH, MODULE_NAME);

        match result {
            Ok(_) => panic!("Expected translation to fail with ReceiveFunctionHasArguments error"),
            Err(CompilationError::ICE(ice_error)) => {
                match ice_error.kind {
                    ICEErrorKind::CompilationContext(
                        CompilationContextError::ReceiveFunctionHasArguments,
                    ) => {
                        // Correct error!
                    }
                    other => panic!("Expected ReceiveFunctionHasArguments, got {other:?}"),
                }
            }
            Err(other) => panic!("Expected ICE error, got {other:?}"),
        }
    }

    #[test]
    fn test_receive_bad_mutability() {
        const MODULE_NAME: &str = "receive_bad_mutability";
        const SOURCE_PATH: &str = "tests/receive/receive_bad_mutability.move";

        let result = translate_test_package_with_framework_result(SOURCE_PATH, MODULE_NAME);

        match result {
            Ok(_) => panic!("Expected translation to fail with ReceiveFunctionIsNotPayable error"),
            Err(CompilationError::ICE(ice_error)) => {
                match ice_error.kind {
                    ICEErrorKind::CompilationContext(
                        CompilationContextError::ReceiveFunctionIsNotPayable,
                    ) => {
                        // Correct error!
                    }
                    other => panic!("Expected ReceiveFunctionIsNotPayable, got {other:?}"),
                }
            }
            Err(other) => panic!("Expected ICE error, got {other:?}"),
        }
    }
}
