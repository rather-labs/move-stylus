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
        assert_eq!(1, result);

        // Assert that the value did not change
        let call_data = readCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(43, return_data);
        assert_eq!(0, result);
    }
}

mod counter_named_id {
    use alloy_primitives::address;
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    // NOTE: we can't use this fixture as #[once] because in order to catch events, we use an mpsc
    // channel. If we use this as #[once], there's a possibility this runtime is used in more than one
    // thread. If that happens, messages from test A can be received by test B.
    // Using once instance per thread assures this won't happen.
    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "counter_named_id";
        const SOURCE_PATH: &str = "tests/storage/counter_named_id.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function create() public view;
        function read() public view returns (uint64);
        function increment() public view;
        function setValue(uint64 value) public view;
    );

    #[rstest]
    fn test_storage_counter_named_id(runtime: RuntimeSandbox) {
        println!("1");
        runtime.print_storage();

        // Create a new counter
        let call_data = createCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        println!("2");
        runtime.print_storage();

        // Read initial value (should be 25)
        let call_data = readCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(25, return_data);
        assert_eq!(0, result);

        runtime.print_storage();
        // Increment
        let call_data = incrementCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(26, return_data);
        assert_eq!(0, result);

        // Set value to 42
        let call_data = setValueCall::new((42,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(42, return_data);
        assert_eq!(0, result);

        // Increment
        let call_data = incrementCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(43, return_data);
        assert_eq!(0, result);

        // change the msg sender
        runtime.set_msg_sender(address!("0x0000000000000000000000000000000abcabcabc").0.0);

        // Set value to 111 with a sender that is not the owner
        let call_data = setValueCall::new((111,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(1, result);

        // Assert that the value did not change
        let call_data = readCall::new(()).abi_encode();
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
    use alloy_primitives::{FixedBytes, U256, address, keccak256};
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

        struct Bar {
            UID id;
            uint64 a;
            uint64[] c;
        }

        struct Qux {
            uint64 a;
            uint128 b;
            uint128 c;
        }

        struct Baz {
            UID id;
            uint64 a;
            Qux c;
        }

        struct Bez {
            UID id;
            uint64 a;
            Qux[] c;
            uint128[][] d;
            uint8 e;
        }

        struct Quz {
            uint64 a;
            uint128 b;
            uint128 c;
        }

        struct Biz {
            UID id;
            uint64 a;
            Quz b;
            Quz[] c;
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
        function createBar() public view;
        function getBar(bytes32 id) public view returns (Bar);
        function deleteBar(bytes32 id) public view;
        function createBaz(address recipient, bool share) public view;
        function getBaz(bytes32 id) public view returns (Baz);
        function deleteBaz(bytes32 id) public view;
        function createBez() public view;
        function getBez(bytes32 id) public view returns (Bez);
        function deleteBez(bytes32 id) public view;
        function createBiz() public view;
        function getBiz(bytes32 id) public view returns (Biz);
        function deleteBiz(bytes32 id) public view;

        function deleteObj2(bytes32 id1, bytes32 id2) public view;
    );

    const SHARED: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    const FROZEN: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
    const COUNTER_KEY: [u8; 32] = [
        88, 181, 235, 71, 20, 200, 162, 193, 179, 99, 195, 177, 236, 158, 218, 42, 168, 26, 11, 70,
        66, 173, 6, 207, 222, 175, 248, 56, 236, 49, 87, 253,
    ];

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

    pub fn get_next_slot(slot: &[u8; 32]) -> [u8; 32] {
        let slot_value = U256::from_be_bytes(*slot);
        (slot_value + U256::from(1)).to_be_bytes()
    }

    // Test create frozen object
    #[rstest]
    fn test_frozen_object(runtime: RuntimeSandbox) {
        let call_data = createFrozenCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Read value
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);
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

        // Delete object
        let call_data = deleteObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
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
    #[should_panic(expected = "unreachable")]
    fn test_delete_frozen_object(runtime: RuntimeSandbox) {
        let call_data = createFrozenCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        // Read value before delete
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Try to delete the object
        let call_data = deleteObjCall::new((object_id,)).abi_encode();
        runtime.call_entrypoint(call_data).unwrap();
    }

    // Test delete owned object
    #[rstest]
    fn test_delete_owned_object(runtime: RuntimeSandbox) {
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);
        let object_slot = derive_object_slot(&SIGNER_ADDRESS, &object_id.0);

        // Read value before delete
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Delete the object
        let call_data = deleteObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the storage from the original slots and check that they are empty
        // Foo takes 2 slots

        // First slot
        assert_eq!(
            [0u8; 32],
            runtime.get_storage_at_slot(object_slot.0),
            "Expected storage value to be 32 zeros"
        );

        // Second slot
        assert_eq!(
            [0u8; 32],
            runtime.get_storage_at_slot(get_next_slot(&object_slot.0)),
            "Expected storage value to be 32 zeros at next slot"
        );
    }

    // Test delete owned object
    #[rstest]
    fn test_delete_shared_object(runtime: RuntimeSandbox) {
        let call_data = createSharedCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);
        let object_slot = derive_object_slot(&SHARED, &object_id.0);

        // Read value before delete
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Delete the object
        let call_data = deleteObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the storage from the original slots and check that they are empty
        // Foo takes 2 slots

        // First slot
        assert_eq!(
            [0u8; 32],
            runtime.get_storage_at_slot(object_slot.0),
            "Expected storage value to be 32 zeros"
        );

        // Second slot
        assert_eq!(
            [0u8; 32],
            runtime.get_storage_at_slot(get_next_slot(&object_slot.0)),
            "Expected storage value to be 32 zeros at next slot"
        );
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

    #[rstest]
    fn test_delete_bar(runtime: RuntimeSandbox) {
        let call_data = createBarCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        let call_data = getBarCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Bar::abi_encode(&Bar {
            id: UID {
                id: ID { bytes: object_id },
            },
            a: 101,
            c: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let call_data = deleteBarCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        for (key, value) in storage_after_delete.iter() {
            if *key != COUNTER_KEY {
                assert!(
                    storage_before_delete.contains_key(key),
                    "Key {:?} should exist in storage_before_delete",
                    key
                );

                assert_eq!(
                    *value, [0u8; 32],
                    "Unexpected non-zero value at key: {:?}",
                    key
                );
            }
        }
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_delete_baz(runtime: RuntimeSandbox, #[case] share: bool) {
        let call_data = createBazCall::new((SIGNER_ADDRESS.into(), share)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        let call_data = getBazCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Baz::abi_encode(&Baz {
            id: UID {
                id: ID { bytes: object_id },
            },
            a: 101,
            c: Qux {
                a: 42,
                b: 55,
                c: 66,
            },
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let call_data = deleteBazCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        for (key, value) in storage_after_delete.iter() {
            if *key != COUNTER_KEY {
                assert!(
                    storage_before_delete.contains_key(key),
                    "Key {:?} should exist in storage_before_delete",
                    key
                );

                assert_eq!(
                    *value, [0u8; 32],
                    "Unexpected non-zero value at key: {:?}",
                    key
                );
            }
        }
    }

    #[rstest]
    fn test_delete_bez(runtime: RuntimeSandbox) {
        let call_data = createBezCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        let call_data = getBezCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Bez::abi_encode(&Bez {
            id: UID {
                id: ID { bytes: object_id },
            },
            a: 101,
            c: vec![
                Qux {
                    a: 42,
                    b: 55,
                    c: 66,
                },
                Qux {
                    a: 43,
                    b: 56,
                    c: 67,
                },
                Qux {
                    a: 44,
                    b: 57,
                    c: 68,
                },
            ],
            d: vec![vec![1, 2, 3], vec![4], vec![], vec![5, 6]],
            e: 17,
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let call_data = deleteBezCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        for (key, value) in storage_after_delete.iter() {
            if *key != COUNTER_KEY {
                // Assert that the key existed in storage before deletion
                assert!(
                    storage_before_delete.contains_key(key),
                    "Key {:?} should exist in storage_before_delete",
                    key
                );

                assert_eq!(
                    *value, [0u8; 32],
                    "Unexpected non-zero value at key: {:?}",
                    key
                );
            }
        }
    }

    #[rstest]
    fn test_delete_biz(runtime: RuntimeSandbox) {
        let call_data = createBizCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        let object_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_id = FixedBytes::<32>::from_slice(&object_id);

        let call_data = getBizCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Biz::abi_encode(&Biz {
            id: UID {
                id: ID { bytes: object_id },
            },
            a: 101,
            b: Quz {
                a: 42,
                b: 55,
                c: 66,
            },
            c: vec![
                Quz {
                    a: 42,
                    b: 55,
                    c: 66,
                },
                Quz {
                    a: 43,
                    b: 56,
                    c: 67,
                },
                Quz {
                    a: 44,
                    b: 57,
                    c: 68,
                },
            ],
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let call_data = deleteBizCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        for (key, value) in storage_after_delete.iter() {
            if *key != COUNTER_KEY {
                // Assert that the key existed in storage before deletion
                assert!(
                    storage_before_delete.contains_key(key),
                    "Key {:?} should exist in storage_before_delete",
                    key
                );

                assert_eq!(
                    *value, [0u8; 32],
                    "Unexpected non-zero value at key: {:?}",
                    key
                );
            }
        }
    }

    #[rstest]
    fn test_delete_many(runtime: RuntimeSandbox) {
        let call_data = createSharedCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_1_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_1_id = FixedBytes::<32>::from_slice(&object_1_id);

        let call_data = createSharedCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_2_id = runtime.log_events.lock().unwrap().recv().unwrap();
        let object_2_id = FixedBytes::<32>::from_slice(&object_2_id);

        let storage_before_delete = runtime.get_storage();

        let call_data = deleteObj2Call::new((object_1_id, object_2_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        for (key, value) in storage_after_delete.iter() {
            if *key != COUNTER_KEY {
                // Assert that the key existed in storage before deletion
                assert!(
                    storage_before_delete.contains_key(key),
                    "Key {:?} should exist in storage_before_delete",
                    key
                );

                assert_eq!(
                    *value, [0u8; 32],
                    "Unexpected non-zero value at key: {:?}",
                    key
                );
            }
        }
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

        // Dynamic structs
        struct DynamicStruct {
            UID id;
            uint32 a;
            bool b;
            uint32[] c;
            uint128[] d;
            uint64 e;
            uint128 f;
            uint256 g;
        }

        struct DynamicStruct2 {
            UID id;
            bool[] a;
            uint8[] b;
            uint16[] c;
            uint32[] d;
            uint64[] e;
            uint128[] f;
            uint256[] g;
            address[] h;
        }

        struct DynamicStruct3 {
            UID id;
            uint8[][] a;
            uint32[][] b;
            uint64[][] c;
            uint128[][] d;
        }

        struct DynamicStruct4 {
            UID id;
            DynamicNestedStructChild[] a;
            StaticNestedStructChild[] b;
        }

        struct DynamicNestedStructChild {
            uint32[] a;
            uint128 b;
        }

        struct NestedStructChildWrapper {
            DynamicNestedStructChild[] a;
            StaticNestedStructChild[] b;
        }

        struct DynamicStruct5 {
            UID id;
            NestedStructChildWrapper[] a;
        }

        struct GenericStruct32 {
            UID id;
            uint32[] a;
            uint32 b;
        }
        function saveDynamicStruct(
            UID id,
            uint32 a,
            bool b,
            uint64[] c,
            uint128[] d,
            uint64 e,
            uint128 f,
            uint256 g,
        ) public view;
        function readDynamicStruct() public view returns (DynamicStruct);

        function saveDynamicStruct2(
            UID id,
            bool[] a,
            uint8[] b,
            uint16[] c,
            uint32[] d,
            uint64[] e,
            uint128[] f,
            uint256[] g,
            address[] h,
        ) public view;
        function readDynamicStruct2() public view returns (DynamicStruct2);

        function saveDynamicStruct3(
            UID id,
            uint8[][] a,
            uint32[][] b,
            uint64[][] c,
            uint128[][] d,
        ) public view;
        function readDynamicStruct3() public view returns (DynamicStruct3);

        function saveDynamicStruct4(
            UID id,
            uint32[] x,
            uint64 y,
            uint128 z,
            address w,
        ) public view;
        function readDynamicStruct4() public view returns (DynamicStruct4);

        function saveDynamicStruct5(
            UID id,
            uint32 x,
            uint64 y,
            uint128 z,
            address w,
        ) public view;
        function readDynamicStruct5() public view returns (DynamicStruct5);

        function saveGenericStruct32(
            UID id,
            uint32 x,
        ) public view;
        function readGenericStruct32() public view returns (GenericStruct32);
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

    #[rstest]
    #[case(saveDynamicStructCall::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        46,
        true,
        vec![2, 3, 4, 5, 6],
        vec![7, 8, 9],
        47,
        48,
        U256::from(49),
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // 0x01
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // 0x02 (vector header)
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(), // vector elements first slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf", 16).unwrap().to_be_bytes(), // vector elements second slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // 0x02 (vector header)
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b", 16).unwrap().to_be_bytes(), // vector elements first slot
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85c", 16).unwrap().to_be_bytes(), // vector elements second slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // 0x04 u64 and u128 slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(), // 0x05 u256 slot

    ],
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("000000000000000000000000000000000000000000000000000000010000002e", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000005000000000000000400000000000000030000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000006", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000800000000000000000000000000000007", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000009", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000000030000000000000002f", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000031", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStructCall::new(()),
        DynamicStruct {
           id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
           a: 46,
           b: true,
           c: vec![2, 3, 4, 5, 6],
           d: vec![7, 8, 9],
           e: 47,
           f: 48,
           g: U256::from(49),
        }
    )]
    #[case(saveDynamicStructCall::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        u32::MAX,
        true,
        vec![],
        vec![7, 8, 9],
        u64::MAX,
        48,
        U256::from(49),
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // 0x01
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // 0x02 (first vector header)
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // 0x03 (second vector header)
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b", 16).unwrap().to_be_bytes(), // vector elements first slot
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85c", 16).unwrap().to_be_bytes(), // vector elements second slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // u64 and u128 slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(), // u256 slot
    ],
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("00000000000000000000000000000000000000000000000000000001ffffffff", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000800000000000000000000000000000007", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000009", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000000000000000000000000000000000000000030ffffffffffffffff", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000031", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStructCall::new(()),
        DynamicStruct {
           id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
           a: u32::MAX,
           b: true,
           c: vec![],
           d: vec![7, 8, 9],
           e: u64::MAX,
           f: 48,
           g: U256::from(49),
        }
    )]
    #[case(saveDynamicStruct2Call::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        vec![true, false, true],
        vec![1, 2, 3, 4, 5], // u8
        vec![6, 7, 8, 9], // u16
        vec![10, 11, 12, 13, 14, 15], // u32
        vec![16, 17, 18, 19, 20], // u64
        vec![21, 22, 23], // u128
        vec![U256::from(24), U256::from(25)], // u256
        vec![address!("0x1111111111111111111111111111111111111111"), address!("0x2222222222222222222222222222222222222222")] // address
    )),
    vec![
        [0x00; 32], // 0x0 UID slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // 0x01 bool vector header
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // bool vec, elem slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // u8 vec, header slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(), // u8 vec, elem slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // u16 vec, header slot
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b", 16).unwrap().to_be_bytes(), // u16 vec, elem slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // u32 vec, header slot
        U256::from_str_radix("8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe36bd19b", 16).unwrap().to_be_bytes(), // u32 vec, elem slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(), // u64 vec, header slot
        U256::from_str_radix("036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db0", 16).unwrap().to_be_bytes(), // u64 vec, elem slot #1
        U256::from_str_radix("036b6384b5eca791c62761152d0c79bb0604c104a5fb6f4eb0703f3154bb3db1", 16).unwrap().to_be_bytes(), // u64 vec, elem slot #2
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000006", 16).unwrap().to_be_bytes(), // u128 vec, header slot
        U256::from_str_radix("f652222313e28459528d920b65115c16c04f3efc82aaedc97be59f3f377c0d3f", 16).unwrap().to_be_bytes(), // u128 vec, elem slot #1
        U256::from_str_radix("f652222313e28459528d920b65115c16c04f3efc82aaedc97be59f3f377c0d40", 16).unwrap().to_be_bytes(), // u128 vec, elem slot #2
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000007", 16).unwrap().to_be_bytes(), // u256 vec, header slot
        U256::from_str_radix("a66cc928b5edb82af9bd49922954155ab7b0942694bea4ce44661d9a8736c688", 16).unwrap().to_be_bytes(), // u256 vec, elem slot #1
        U256::from_str_radix("a66cc928b5edb82af9bd49922954155ab7b0942694bea4ce44661d9a8736c689", 16).unwrap().to_be_bytes(), // u256 vec, elem slot #2
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000008", 16).unwrap().to_be_bytes(), // address vec, header slot
        U256::from_str_radix("f3f7a9fe364faab93b216da50a3214154f22a0a2b415b23a84c8169e8b636ee3", 16).unwrap().to_be_bytes(), // address vec, elem slot #1
        U256::from_str_radix("f3f7a9fe364faab93b216da50a3214154f22a0a2b415b23a84c8169e8b636ee4", 16).unwrap().to_be_bytes(), // address vec, elem slot #2
    ],
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000010001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000504030201", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000009000800070006", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000006", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000000000000000000f0000000e0000000d0000000c0000000b0000000a", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000013000000000000001200000000000000110000000000000010", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000014", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000001600000000000000000000000000000015", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000017", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000018", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000019", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000001111111111111111111111111111111111111111", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000002222222222222222222222222222222222222222", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStruct2Call::new(()),
        DynamicStruct2 {
           id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
           a: vec![true, false, true],
           b: vec![1, 2, 3, 4, 5],
           c: vec![6, 7, 8, 9],
           d: vec![10, 11, 12, 13, 14, 15],
           e: vec![16, 17, 18, 19, 20],
           f: vec![21, 22, 23],
           g: vec![U256::from(24), U256::from(25)],
           h: vec![address!("0x1111111111111111111111111111111111111111"), address!("0x2222222222222222222222222222222222222222")],
        }
    )]
    #[case(saveDynamicStruct3Call::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        vec![vec![1, 2, 3], vec![4, 5]],
        vec![vec![6, 7], vec![8], vec![9, 10]],
        vec![vec![11, 12, 13, 14], vec![], vec![15, 16]],
        vec![vec![17, 18, 19]],
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // 0x01 (u8[][] header) slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // first u8[] header slot
        U256::from_str_radix("b5d9d894133a730aa651ef62d26b0ffa846233c74177a591a4a896adfda97d22", 16).unwrap().to_be_bytes(), // first u8[] elements slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf7", 16).unwrap().to_be_bytes(), // second u8[] header slot
        U256::from_str_radix("ea7809e925a8989e20c901c4c1da82f0ba29b26797760d445a0ce4cf3c6fbd31", 16).unwrap().to_be_bytes(), // second u8[] elements slot

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // 0x02 (u32[][] header) slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(), // first u32[] header slot
        U256::from_str_radix("1ab0c6948a275349ae45a06aad66a8bd65ac18074615d53676c09b67809099e0", 16).unwrap().to_be_bytes(), // first u32[] elements slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf", 16).unwrap().to_be_bytes(), // second u32[] header slot
        U256::from_str_radix("2f2149d90beac0570c7f26368e4bc897ca24bba51b1a0f4960d358f764f11f31", 16).unwrap().to_be_bytes(), // second u32[] elements slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ad0", 16).unwrap().to_be_bytes(), // third u32[] header slot
        U256::from_str_radix("4aee6d38ad948303a0117a3e3deee4d912b62481681bd892442a7d720eee5d2c", 16).unwrap().to_be_bytes(), // third u32[] elements slot

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // 0x03 (u64[][] header) slot
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b", 16).unwrap().to_be_bytes(), // first u64[] header slot
        U256::from_str_radix("2584db4a68aa8b172f70bc04e2e74541617c003374de6eb4b295e823e5beab01", 16).unwrap().to_be_bytes(), // first u64[] elements slot
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85c", 16).unwrap().to_be_bytes(), // second u64[] header slot (empty vector)
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85d", 16).unwrap().to_be_bytes(), // third u64[] header slot
        U256::from_str_radix("3f8a9ffd58db029f2bac46056dbc53052839d91105f501f2db6ecb9566ee6832", 16).unwrap().to_be_bytes(), // third u64[] elements slot

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // 0x04 (u128[][] header) slot
        U256::from_str_radix("8a35acfbc15ff81a39ae7d344fd709f28e8600b4aa8c65c6b64bfe7fe36bd19b", 16).unwrap().to_be_bytes(), // u128[] header slot
        U256::from_str_radix("c167b0e3c82238f4f2d1a50a8b3a44f96311d77b148c30dc0ef863e1a060dcb6", 16).unwrap().to_be_bytes(), // u128[] elements slot #1
        U256::from_str_radix("c167b0e3c82238f4f2d1a50a8b3a44f96311d77b148c30dc0ef863e1a060dcb7", 16).unwrap().to_be_bytes(), // u128[] elements slot #2
    ],
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // u32[][] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // first u8[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000030201", 16).unwrap().to_be_bytes(), // first u8[] elements
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // second u8[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000504", 16).unwrap().to_be_bytes(), // second u8[] elements

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // u32[][] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // first u32[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000700000006", 16).unwrap().to_be_bytes(), // first u32[] elements
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // second u32[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000008", 16).unwrap().to_be_bytes(), // second u32[] elements
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // third u32[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000a00000009", 16).unwrap().to_be_bytes(), // third u32[] elements

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // u64[][] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // first u64[] len
        U256::from_str_radix("000000000000000e000000000000000d000000000000000c000000000000000b", 16).unwrap().to_be_bytes(), // first u64[] elements
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(), // second u64[] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // third u64[] len
        U256::from_str_radix("000000000000000000000000000000000000000000000010000000000000000f", 16).unwrap().to_be_bytes(), // third u64[] elements

        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // u128[][] len
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // first u128[] len
        U256::from_str_radix("0000000000000000000000000000001200000000000000000000000000000011", 16).unwrap().to_be_bytes(), // u128[] elements #1
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000013", 16).unwrap().to_be_bytes(), // u128[] elements #2
    ],
        readDynamicStruct3Call::new(()),
        DynamicStruct3 {
           id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
           a: vec![vec![1, 2, 3], vec![4, 5]],
           b: vec![vec![6, 7], vec![8], vec![9, 10]],
           c: vec![vec![11, 12, 13, 14], vec![], vec![15, 16]],
           d: vec![vec![17, 18, 19]],
        }
    )]
    #[case(saveDynamicStruct4Call::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        vec![1, 2, 3],
        47,
        123,
        address!("0x1111111111111111111111111111111111111111"),
    )),
    vec![
        // Field uid
        [0x00; 32], // 0x0
        // Field a: DynamicNestedStructChild[]
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // Header slot
        // First element
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // uint32[] header slot
        U256::from_str_radix("b5d9d894133a730aa651ef62d26b0ffa846233c74177a591a4a896adfda97d22", 16).unwrap().to_be_bytes(), // uint32[] elements slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf7", 16).unwrap().to_be_bytes(), // u128
        // Second element
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf8", 16).unwrap().to_be_bytes(), // uint32[] header slot
        U256::from_str_radix("b32787652f8eacc66cda8b4b73a1b9c31381474fe9e723b0ba866bfbd5dde02b", 16).unwrap().to_be_bytes(), // uint32[] elements slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf9", 16).unwrap().to_be_bytes(), // u128

        // Field b: StaticNestedStructChild[]
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // Header slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(), // First element
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf", 16).unwrap().to_be_bytes(), // Second element
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ad0", 16).unwrap().to_be_bytes(), // Third element
    ],
    vec![
        [0x00; 32], // 0x0
        // Field a: DynamicNestedStructChild[]
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        // First element
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000030000000200000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000000000000000000000007b", 16).unwrap().to_be_bytes(),
        // Second element
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000030000000200000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000000000000000000000007c", 16).unwrap().to_be_bytes(),
        // Field b: StaticNestedStructChild[]
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000001111111111111111111111111111111111111111000000000000002f", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000011111111111111111111111111111111111111110000000000000030", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000011111111111111111111111111111111111111110000000000000031", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStruct4Call::new(()),
        DynamicStruct4 {
           id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
           a: vec![DynamicNestedStructChild { a: vec![1, 2, 3], b: 123 }, DynamicNestedStructChild { a: vec![1, 2, 3], b: 124 }],
           b: vec![StaticNestedStructChild { d: 47, e: address!("0x1111111111111111111111111111111111111111") }, StaticNestedStructChild { d: 48, e: address!("0x1111111111111111111111111111111111111111") }, StaticNestedStructChild { d: 49, e: address!("0x1111111111111111111111111111111111111111") }],
        }
    )]
    #[case(saveDynamicStruct5Call::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        1,
        42,
        123,
        address!("0x1111111111111111111111111111111111111111"),
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // Header slot
    ],
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStruct5Call::new(()),
        DynamicStruct5 {
           id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
           a: vec![
               NestedStructChildWrapper {
                   a: vec![
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 123 },
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 124 }
                   ],
                   b: vec![
                       StaticNestedStructChild { d: 42, e: address!("0x1111111111111111111111111111111111111111") },
                       StaticNestedStructChild { d: 43, e: address!("0x1111111111111111111111111111111111111111") },
                       StaticNestedStructChild { d: 44, e: address!("0x1111111111111111111111111111111111111111") }
                   ]
               },
               NestedStructChildWrapper {
                   a: vec![
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 125 },
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 126 },
                       DynamicNestedStructChild { a: vec![1, 2, 3], b: 127 }
                   ],
                   b: vec![
                       StaticNestedStructChild { d: 45, e: address!("0x1111111111111111111111111111111111111111") },
                       StaticNestedStructChild { d: 46, e: address!("0x1111111111111111111111111111111111111111") },
                   ]
               }
           ],
        }
    )]
    #[case(saveGenericStruct32Call::new((
        UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
        1,
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // uint32[] header slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // uint32[] elements slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // uint32 b
    ],
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // Header slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000030000000200000001", 16).unwrap().to_be_bytes(), // First element
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // Second element
    ],
        readGenericStruct32Call::new(()),
        GenericStruct32 {
            id: UID { id: ID { bytes: address!("0x0000000000000000000000000000000000000000") } },
            a: vec![1, 2, 3],
            b: 1,
        }
    )]
    fn test_dynamic_fields<T: SolCall, U: SolCall, V: SolValue>(
        runtime: RuntimeSandbox,
        #[case] call_data_encode: T,
        #[case] expected_slots: Vec<[u8; 32]>,
        #[case] expected_encode: Vec<[u8; 32]>,
        #[case] call_data_decode: U,
        #[case] expected_decode: V,
    ) {
        let (result, _) = runtime
            .call_entrypoint(call_data_encode.abi_encode())
            .unwrap();
        assert_eq!(0, result);

        // Check if it is encoded correctly in storage
        for (i, slot) in expected_slots.iter().enumerate() {
            let storage = runtime.get_storage_at_slot(*slot);
            assert_eq!(expected_encode[i], storage, "Mismatch at slot {}", i);
        }

        // Use the read function to check if it decodes correctly
        let (result, result_data) = runtime
            .call_entrypoint(call_data_decode.abi_encode())
            .unwrap();
        assert_eq!(0, result);
        assert_eq!(expected_decode.abi_encode(), result_data);
    }
}
