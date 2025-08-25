mod common;

use common::runtime_sandbox::constants::SIGNER_ADDRESS;
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

    use crate::common::runtime_sandbox::constants::SIGNER_ADDRESS;

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

mod storage_transfer {
    use alloy_primitives::{FixedBytes, address, keccak256};
    use alloy_sol_types::{SolCall, SolValue, sol};

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

        #[derive(Debug)]
        struct ID {
           bytes32 bytes;
        }

        #[derive(Debug)]
        struct UID {
           ID id;
        }

        struct Foo {
            UID id;
            uint64 value;
        }

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
        function getFoo(bytes32 id) public view returns (Foo);
    );

    const SHARED: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    const FROZEN: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];

    /// Right-align `data` into a 32-byte word (EVM storage encoding for value types).
    #[inline]
    fn pad32_right(data: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        let n = data.len().min(32);
        out[32 - n..].copy_from_slice(&data[..n]); // <-- right-align
        out
    }

    /// mapping(address => mapping(bytes32 => V)) at base slot 0
    /// slot(owner, id) = keccak256( pad32(id) || keccak256( pad32(owner) || pad32(0) ) )
    pub fn derive_object_slot(owner: &[u8], object_id: &[u8]) -> FixedBytes<32> {
        // parent = keccak256( pad32(owner) || pad32(0) )
        let owner_padded = pad32_right(owner);
        let zero_slot = [0u8; 32];

        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(&owner_padded);
        buf[32..].copy_from_slice(&zero_slot);
        let parent = keccak256(buf);

        // slot = keccak256( pad32(id) || pad32(parent) )
        let id_padded = pad32_right(object_id); // object_id is already 32B, this is a no-op
        buf[..32].copy_from_slice(&id_padded);
        buf[32..].copy_from_slice(parent.as_slice());
        keccak256(buf)
    }

    // Tests operations on a shared object: reading, updating values, etc.
    #[rstest]
    fn test_shared_object(runtime: RuntimeSandbox) {
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

        // Change the signer
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

        // Change the signer
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000acacacac").0.0);

        // Increment
        let call_data = incrementValueCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Assert that the value did not change
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(112, return_data);
        assert_eq!(0, result);

        // Change the msg sender
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

        // Change the msg sender
        // Should still work since the signer is the owner
        runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);

        // Set value to 111 with a sender that is not the owner
        let call_data = setValueCall::new((object_id, 111)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Assert that the value was changes correctly
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(111, return_data);
        assert_eq!(0, result);
    }

    // Tests the share of an object in both owned and shared cases.
    #[rstest]
    fn test_share_owned_object(runtime: RuntimeSandbox) {
        // Create a new object
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Compute the object slot using the owner and the object id
        let owner = runtime.get_tx_origin();
        let object_slot = derive_object_slot(&owner, &object_id.0);

        // Read the storage on the original slot before the freeze
        let value_before_share = runtime.get_storage_at_slot(object_slot.0);

        // Share the object. Only possible if the object is owned by the signer!
        let call_data = shareObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the storage on the original slot after the share
        // Should be zeroes since the object moved from the owner space to the shared space
        let value_after_share = runtime.get_storage_at_slot(object_slot.0);
        assert_eq!(
            [0u8; 32], value_after_share,
            "Expected storage value to be 32 zeros"
        );

        // Get the slot number for the shared object
        let shared_slot = derive_object_slot(&SHARED, &object_id.0);

        // Read the storage on the shared slot after the share
        // Should be the same as the original slot before the share
        let shared_value = runtime.get_storage_at_slot(shared_slot.0);
        assert_eq!(
            value_before_share, shared_value,
            "Expected storage value to be the same"
        );

        // Read value
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Change the signer and read again
        // Should still work since the object is shared
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);
    }

    // Tests the freeze of an object in both owned case.
    #[rstest]
    fn test_freeze_owned_object(runtime: RuntimeSandbox) {
        // Create a new object
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Compute the object slot using the owner and the object id
        let owner = runtime.get_tx_origin();
        let object_slot = derive_object_slot(&owner, &object_id.0);

        // Read the storage on the original slot before the freeze
        let value_before_freeze = runtime.get_storage_at_slot(object_slot.0);

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the storage on the original slot after the freeze
        // Should be zeroes since the object moved from the owner space to the frozen space
        let value_after_freeze = runtime.get_storage_at_slot(object_slot.0);
        assert_eq!(
            [0u8; 32], value_after_freeze,
            "Expected storage value to be 32 zeros"
        );

        // Compute the object slot using the FROZEN address and the object id
        let frozen_slot = derive_object_slot(&FROZEN, &object_id.0);

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
        // Should still work since the object is frozen
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Change the msg sender and read again
        // Should still work since the object is frozen
        runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
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
        runtime.call_entrypoint(call_data).unwrap();
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

        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new((object_id,)).abi_encode();
        runtime.call_entrypoint(call_data).unwrap();
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

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new((object_id,)).abi_encode();
        runtime.call_entrypoint(call_data).unwrap();
    }

    // Freeze and then try to share or transfer the object.
    #[rstest]
    #[should_panic(expected = "unreachable")]
    #[case(false)]
    #[should_panic(expected = "unreachable")]
    #[case(true)]
    fn test_share_or_transfer_frozen(runtime: RuntimeSandbox, #[case] share: bool) {
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        if share {
            // Try to share the object.
            let call_data = shareObjCall::new((object_id,)).abi_encode();
            runtime.call_entrypoint(call_data).unwrap();
        } else {
            // Try to transfer the object.
            let call_data = transferObjCall::new((object_id, SIGNER_ADDRESS.into())).abi_encode();
            runtime.call_entrypoint(call_data).unwrap();
        }
    }

    #[rstest]
    fn test_get_foo(runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Set value to 111 with a sender that is not the owner
        let call_data = getFooCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Foo::abi_encode(&Foo {
            id: UID {
                id: ID { bytes: object_id },
            },
            value: 101,
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);
    }
}

mod storage_encoding {
    use alloy_primitives::{U256, address};
    use alloy_sol_types::SolValue;
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    // NOTE: we can't use this fixture as #[once] because in order to catch events, we use an mpsc
    // channel. If we use this as #[once], there's a possibility this runtime is used in more than one
    // thread. If that happens, messages from test A can be received by test B.
    // Using once instance per thread assures this won't happen.
    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "storage_encoding";
        const SOURCE_PATH: &str = "tests/storage/encoding.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]

        #[derive(Debug)]
        struct ID {
           address bytes;
        }

        #[derive(Debug)]
        struct UID {
           ID id;
        }

        struct StaticFields {
            UID id;
            uint256 a;
            uint128 b;
            uint64 c;
            uint32 d;
            uint16 e;
            uint8 f;
            address g;
        }

        struct StaticFields2 {
            UID id;
            uint8 a;
            address b;
            uint64 c;
            uint16 d;
            uint8 e;
        }

        struct StaticFields3 {
            UID id;
            uint8 a;
            address b;
            uint64 c;
            address d;
        }

        struct StaticNestedStruct {
            UID id;
            uint64 a;
            bool b;
            StaticNestedStructChild c;
            uint128 f;
            uint32 g;
        }

        struct StaticNestedStructChild {
            uint64 d;
            address e;
        }

        function saveStaticFields(
            UID id,
            uint256 a,
            uint128 b,
            uint64 c,
            uint32 d,
            uint16 e,
            uint8 f,
            address g
        ) public view;
        function readStaticFields() public view returns (StaticFields);

        function saveStaticFields2(
            UID id,
            uint8 a,
            address b,
            uint64 c,
            uint16 d,
            uint8 e
        ) public view;
        function readStaticFields2() public view returns (StaticFields2);

        function saveStaticFields3(
            UID id,
            uint8 a,
            address b,
            uint64 c,
            address d
        ) public view;
        function readStaticFields3() public view returns (StaticFields3);

        function saveStaticNestedStruct(
            UID id,
            uint64 a,
            bool b,
            uint64 d,
            address e,
            uint128 f,
            uint32 g
        ) public view;
        function readStaticNestedStruct() public view returns (StaticNestedStruct);
    );

    #[rstest]
    #[case(saveStaticFieldsCall::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        U256::from_str_radix("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 16).unwrap(),
        0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        0xcccccccccccccccc,
        0xdddddddd,
        0xeeee,
        0xff,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
    )), vec![
        [0x00; 32],
        [0xaa; 32],
        U256::from_str_radix("ffeeeeddddddddccccccccccccccccbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
    ],
        readStaticFieldsCall::new(()),
        StaticFields {
            id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
            a: U256::from_str_radix("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 16).unwrap(),
            b: 0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
            c: 0xcccccccccccccccc,
            d: 0xdddddddd,
            e: 0xeeee,
            f: 0xff,
            g: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        }
    )]
    #[case(saveStaticFieldsCall::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        U256::from(1),
        2,
        3,
        4,
        5,
        6,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
    )), vec![
        [0x00; 32],
        U256::from(1).to_be_bytes(),
        U256::from_str_radix("06000500000004000000000000000300000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
    ],
        readStaticFieldsCall::new(()),
        StaticFields {
            id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
            a: U256::from(1),
            b: 2,
            c: 3,
            d: 4,
            e: 5,
            f: 6,
            g: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        }
    )]
    #[case(saveStaticFields2Call::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        0xff,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        0xcccccccccccccccc,
        0xeeee,
        0xff,
    )), vec![
        [0x00; 32],
        U256::from_str_radix("ffeeeecccccccccccccccccafecafecafecafecafecafecafecafecafecafeff", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields2Call::new(()),
        StaticFields2 {
            id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
            a: 0xff,
            b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
            c: 0xcccccccccccccccc,
            d: 0xeeee,
            e: 0xff,
        }
    )]
    #[case(saveStaticFields2Call::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        1,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        2,
        3,
        4,
    )), vec![
        [0x00; 32],
        U256::from_str_radix("0400030000000000000002cafecafecafecafecafecafecafecafecafecafe01", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields2Call::new(()),
        StaticFields2 {
            id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
            a: 1,
            b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
            c: 2,
            d: 3,
            e: 4,
        }
    )]
    #[case(saveStaticFields3Call::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        1,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        2,
        address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
    )), vec![
        [0x00; 32],
        U256::from_str_radix("0000000000000000000002cafecafecafecafecafecafecafecafecafecafe01", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000beefbeefbeefbeefbeefbeefbeefbeefbeefbeef", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields3Call::new(()),
        StaticFields3 {
           id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
           a: 1,
           b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
           c: 2,
           d: address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
        }
    )]
    #[case(saveStaticFields3Call::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        0xff,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        0xcccccccccccccccc,
        address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
    )), vec![
        [0x00; 32],
        U256::from_str_radix("000000cccccccccccccccccafecafecafecafecafecafecafecafecafecafeff", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000beefbeefbeefbeefbeefbeefbeefbeefbeefbeef", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields3Call::new(()),
        StaticFields3 {
            id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
            a: 0xff,
            b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
            c: 0xcccccccccccccccc,
            d: address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
        }
    )]
    #[case(saveStaticNestedStructCall::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        1,
        true,
        2,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        3,
        4
    )), vec![
        [0x00; 32],
        U256::from_str_radix("0000000000000002010000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000400000000000000000000000000000003", 16).unwrap().to_be_bytes(),
    ],
        readStaticNestedStructCall::new(()),
        StaticNestedStruct {
           id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
           a: 1,
           b: true,
           c: StaticNestedStructChild {
                d: 2,
                e: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
           },
           f: 3,
           g: 4,
        }
    )]
    #[case(saveStaticNestedStructCall::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        0xaaaaaaaaaaaaaaaa,
        true,
        0xbbbbbbbbbbbbbbbb,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        0xcccccccccccccccccccccccccccccccc,
        0xdddddddd,
    )), vec![
        [0x00; 32],
        U256::from_str_radix("000000000000000000000000000000bbbbbbbbbbbbbbbb01aaaaaaaaaaaaaaaa", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000ddddddddcccccccccccccccccccccccccccccccc", 16).unwrap().to_be_bytes(),
    ],
        readStaticNestedStructCall::new(()),
        StaticNestedStruct {
           id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
           a: 0xaaaaaaaaaaaaaaaa,
           b: true,
           c: StaticNestedStructChild {
                d: 0xbbbbbbbbbbbbbbbb,
                e: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
           },
           f: 0xcccccccccccccccccccccccccccccccc,
           g: 0xdddddddd,
        }
    )]
    fn test_static_fields<T: SolCall, U: SolCall, V: SolValue>(
        runtime: RuntimeSandbox,
        #[case] call_data_encode: T,
        #[case] expected_encode: Vec<[u8; 32]>,
        #[case] call_data_decode: U,
        #[case] expected_decode: V,
    ) {
        let (result, _) = runtime
            .call_entrypoint(call_data_encode.abi_encode())
            .unwrap();
        assert_eq!(0, result);

        // Check if it is encoded correctly in storage
        for (i, expected) in expected_encode.iter().enumerate() {
            let storage = runtime.get_storage_at_slot(U256::from(i).to_be_bytes());
            assert_eq!(expected, &storage, "Mismatch at slot {}", i);
        }

        // Use the read function to check if it decodes correctly
        let (result, result_data) = runtime
            .call_entrypoint(call_data_decode.abi_encode())
            .unwrap();
        assert_eq!(0, result);
        assert_eq!(expected_decode.abi_encode(), result_data);
    }
}
