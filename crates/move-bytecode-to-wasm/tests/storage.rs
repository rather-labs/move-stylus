mod common;

use common::runtime_sandbox::constants::SIGNER_ADDRESS;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package_with_framework};
use rstest::{fixture, rstest};

mod storage_counter {
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

mod storage_transfer {
    use alloy_primitives::{FixedBytes, address};
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "transfer";
        const SOURCE_PATH: &str = "tests/storage/transfer.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function createShared() public view;
        function createOwned(address recipient) public view;
        function createFrozen() public view;
        function readValue(bytes32 id) public view returns (uint64);
        function setValue(bytes32 id, uint64 value) public view;
        function incrementValue(bytes32 id) public view;
        function deleteObj(bytes32 id) public view;
        function freezeObj(bytes32 id) public view;
        function shareObj(bytes32 id) public view;
        function transferObj(bytes32 id, address recipient) public view;
    );

    // Tests operations on a shared object: reading, updating values, etc.
    #[rstest]
    fn test_shared_object(runtime: RuntimeSandbox) {
        // TODO: remove this.
        runtime.set_msg_sender(SIGNER_ADDRESS);

        // Create a new counter
        let call_data = createSharedCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Read initial value (should be 101)
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Increment
        let call_data = incrementValueCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(102, return_data);
        assert_eq!(0, result);

        // Set value to 42
        let call_data = setValueCall::new((object_id, 42)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(42, return_data);
        assert_eq!(0, result);

        // Increment
        let call_data = incrementValueCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(43, return_data);
        assert_eq!(0, result);

        // change the signer
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

        // Set value to 111
        let call_data = setValueCall::new((object_id, 111)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Assert that the value is set
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(111, return_data);
        assert_eq!(0, result);

        // change the signer
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000acacacac").0.0);

        // Set value to 111
        let call_data = incrementValueCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Assert that the value did not change
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(112, return_data);
        assert_eq!(0, result);

        // change the msg sender
        runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);

        // Set value to 1111 with a sender that is not the owner
        let call_data = setValueCall::new((object_id, 1111)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(1111, return_data);
        assert_eq!(0, result);
    }

    // Tests operations on an owned object: reading, updating values, etc.
    #[rstest]
    fn test_owned_object(runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Read initial value (should be 101)
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Increment
        let call_data = incrementValueCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(102, return_data);
        assert_eq!(0, result);

        // Set value to 42
        let call_data = setValueCall::new((object_id, 42)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(42, return_data);
        assert_eq!(0, result);

        // Increment
        let call_data = incrementValueCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(43, return_data);
        assert_eq!(0, result);

        // change the msg sender
        runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);

        // Set value to 111 with a sender that is not the owner
        let call_data = setValueCall::new((object_id, 111)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Assert that the value did not change
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(111, return_data);
        assert_eq!(0, result);
    }

    // Tests trying to read an owned object with a signer that is not the owner.
    #[rstest]
    #[should_panic(expected = "unreachable")]
    fn test_signer_owner_mismatch(runtime: RuntimeSandbox) {
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Read initial value (should be 101)
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // change the signer
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

        // This should hit an unreachable due to the signer differing from the owner!
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
    }

    // Tests the freeze of an object in both owned and shared cases.
    #[rstest]
    fn test_freeze_owned_object(runtime: RuntimeSandbox) {
        // Create a new object
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Read the object slot emmited from the contract's events
        let object_slot = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_slot = FixedBytes::<32>::from_slice(&object_slot);

        // Read the storage on the original slot before the freeze
        let value_before_freeze = runtime.get_storage_at_slot(object_slot.0);

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the storage on the original slot after the freeze
        let value_after_freeze = runtime.get_storage_at_slot(object_slot.0);
        assert_eq!(
            [0u8; 32], value_after_freeze,
            "Expected storage value to be 32 zeros"
        );

        // Read the object id emmited from the contract's events
        let frozen_slot = runtime.log_events.lock().unwrap().recv().unwrap();
        let frozen_slot = FixedBytes::<32>::from_slice(&frozen_slot);

        // Read the storage on the frozen slot after the freeze
        let frozen_value = runtime.get_storage_at_slot(frozen_slot.0);
        assert_eq!(
            value_before_freeze, frozen_value,
            "Expected storage value to be the same"
        );

        // Read value
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Change the signer and read again
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);
    }

    // Tests the freeze of an object that is not owned by the signer.
    #[rstest]
    #[should_panic(expected = "unreachable")]
    fn test_freeze_not_owned_object(runtime: RuntimeSandbox) {
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Read the object slot emmited from the contract's events
        let object_slot = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_slot = FixedBytes::<32>::from_slice(&object_slot);

        // Read the storage on the original slot before the freeze
        let value_before_freeze = runtime.get_storage_at_slot(object_slot.0);

        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }

    // Tests the freeze of a shared object.
    #[rstest]
    #[should_panic(expected = "unreachable")]
    fn test_freeze_shared_object(runtime: RuntimeSandbox) {
        // Create a new object
        let call_data = createSharedCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Read the object slot emmited from the contract's events
        let object_slot = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_slot = FixedBytes::<32>::from_slice(&object_slot);

        // Read the storage on the original slot before the freeze
        let value_before_freeze = runtime.get_storage_at_slot(object_slot.0);

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }

    // Freeze and then try to share or transfer the object.
    #[rstest]
    #[should_panic(expected = "unreachable")]
    #[case(false)]
    #[should_panic(expected = "unreachable")]
    #[case(true)]
    fn test_share_or_transfer_after_freeze(runtime: RuntimeSandbox, #[case] share: bool) {
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Read the object slot emmited from the contract's events
        let object_slot = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_slot = FixedBytes::<32>::from_slice(&object_slot);

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        if share {
            // Try to share the object.
            let call_data = shareObjCall::new((object_id,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);
        } else {
            // Try to transfer the object.
            let call_data = transferObjCall::new((object_id, SIGNER_ADDRESS.into())).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);
        }
    }
}
