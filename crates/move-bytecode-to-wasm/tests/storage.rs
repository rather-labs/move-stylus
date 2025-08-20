mod common;

use common::{runtime_sandbox::RuntimeSandbox, translate_test_package_with_framework};
use rstest::{fixture, rstest};

mod counter {
    use alloy_primitives::{FixedBytes, address};
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    // NOTE: we can't use this fixture as #[once] because in order to catch events, we use an mpsc
    // channel. If we use this as #[once], there's a possibility this runtime is used in more than one
    // thread. If that happens, messages from test A can be received by test B.
    // Using once instance per thread assures this won't happen.
    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "counter";
        const SOURCE_PATH: &str = "tests/storage/counter.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function create() public view;
        function read(bytes32 id) public view returns (uint64);
        function increment(bytes32 id) public view;
        function setValue(bytes32 id, uint64 value) public view;
    );

    #[rstest]
    fn test_storage_counter(runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = createCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

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
        assert_eq!(0, result);

        // Assert that the value did not change
        let call_data = readCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(43, return_data);
        assert_eq!(0, result);
    }
}

mod capability {
    use alloy_primitives::{FixedBytes, address};
    use alloy_sol_types::{SolCall, sol};

    use crate::common::runtime_sandbox::constants::{MSG_SENDER_ADDRESS, SIGNER_ADDRESS};

    use super::*;

    // NOTE: we can't use this fixture as #[once] because in order to catch events, we use an mpsc
    // channel. If we use this as #[once], there's a possibility this runtime is used in more than one
    // thread. If that happens, messages from test A can be received by test B.
    // Using once instance per thread assures this won't happen.
    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "capability";
        const SOURCE_PATH: &str = "tests/storage/capability.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function create() public view;
        function adminCapFn(bytes32 id) public view;
    );

    #[rstest]
    fn test_capability(runtime: RuntimeSandbox) {
        // Set the sender as the signer, because the owner will be the sender (and we are sending
        // the transaction from the same address that signs it)
        runtime.set_msg_sender(SIGNER_ADDRESS);

        // Create a new counter
        let call_data = createCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

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
}
