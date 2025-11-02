mod common;

use alloy_primitives::hex;
use alloy_primitives::{FixedBytes, U256, keccak256};
use common::runtime_sandbox::constants::SIGNER_ADDRESS;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package_with_framework};
use rstest::{fixture, rstest};

const SHARED: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
const FROZEN: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
const COUNTER_KEY: [u8; 32] = [
    88, 181, 235, 71, 20, 200, 162, 193, 179, 99, 195, 177, 236, 158, 218, 42, 168, 26, 11, 70, 66,
    173, 6, 207, 222, 175, 248, 56, 236, 49, 87, 253,
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

// Helper function to assert that all storage slots are empty after a delete operation
// It checks that keys from before_delete now have zero values in after_delete
pub fn assert_empty_storage(
    storage_before_delete: &std::collections::HashMap<[u8; 32], [u8; 32]>,
    storage_after_delete: &std::collections::HashMap<[u8; 32], [u8; 32]>,
) {
    // Check that keys that existed before delete now have zero values after delete, except for the counter key
    for key in storage_before_delete.keys() {
        if *key != COUNTER_KEY {
            let value_after = storage_after_delete.get(key).unwrap_or(&[0u8; 32]);
            assert_eq!(
                *value_after, [0u8; 32],
                "Unexpected non-zero value at key {:?} after delete",
                key
            );
        }
    }
}

mod counter {
    use alloy_primitives::address;
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
        let object_id = runtime.obtain_uid();

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
        // Create a new counter
        let call_data = createCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read initial value (should be 25)
        let call_data = readCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(25, return_data);
        assert_eq!(0, result);

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
    use alloy_primitives::address;
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
}

mod storage_transfer_named_id {
    use alloy_primitives::address;
    use alloy_sol_types::{SolCall, SolValue, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "transfer_named_id";
        const SOURCE_PATH: &str = "tests/storage/transfer_named_id.move";

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
        struct NamedId {
           ID id;
        }

        struct Foo {
            NamedId id;
            uint64 value;
        }

        struct Bar {
            NamedId id;
            uint64 a;
            uint64[] c;
        }

        struct Qux {
            uint64 a;
            uint128 b;
            uint128 c;
        }

        struct Baz {
            NamedId id;
            uint64 a;
            Qux c;
        }

        struct Bez {
            NamedId id;
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
            NamedId id;
            uint64 a;
            Quz b;
            Quz[] c;
        }


        #[allow(missing_docs)]
        function createShared() public view;
        function createOwned(address recipient) public view;
        function createFrozen() public view;
        function readValue() public view returns (uint64);
        function setValue(uint64 value) public view;
        function incrementValue() public view;
        function deleteObj() public view;
        function freezeObj() public view;
        function shareObj() public view;
        function transferObj(address recipient) public view;
        function getFoo() public view returns (Foo);
        function createBar() public view;
        function getBar() public view returns (Bar);
        function deleteBar() public view;
        function createBaz(address recipient, bool share) public view;
        function getBaz() public view returns (Baz);
        function deleteBaz() public view;
        function createBez() public view;
        function getBez() public view returns (Bez);
        function deleteBez() public view;
        function createBiz() public view;
        function getBiz() public view returns (Biz);
        function deleteBiz() public view;
    );

    const FOO_ID: [u8; 32] = [
        0x04, 0xd9, 0x56, 0x9c, 0xa9, 0x35, 0xbc, 0x8c, 0x4d, 0x83, 0x5e, 0xf7, 0x49, 0xba, 0x26,
        0x04, 0x0d, 0x5a, 0x5a, 0xbc, 0x33, 0xe3, 0xe3, 0x4e, 0x4a, 0x9e, 0xe6, 0x91, 0xe2, 0x93,
        0x4f, 0xc7,
    ];
    const BAR_ID: [u8; 32] = [
        0x07, 0xcc, 0xac, 0x83, 0x2b, 0x4b, 0x6e, 0x44, 0x05, 0x80, 0xae, 0x89, 0x26, 0xba, 0xcf,
        0x74, 0xae, 0xe4, 0xf1, 0x90, 0x78, 0x2a, 0x69, 0xed, 0x94, 0x80, 0xee, 0x90, 0xd2, 0xac,
        0x90, 0x87,
    ];
    const BAZ_ID: [u8; 32] = [
        0xcc, 0xd6, 0xf0, 0x70, 0x9a, 0xda, 0xce, 0xfb, 0xfc, 0x3b, 0x75, 0x15, 0x74, 0x62, 0x0a,
        0xf6, 0x39, 0xb0, 0x09, 0x14, 0x44, 0x16, 0x68, 0x40, 0xd7, 0x02, 0x9b, 0x05, 0x10, 0x9d,
        0x69, 0xa0,
    ];
    const BEZ_ID: [u8; 32] = [
        0xdb, 0xb5, 0x11, 0xf8, 0x0e, 0x54, 0xba, 0xb0, 0x5d, 0x9e, 0xa1, 0xbf, 0x80, 0xca, 0xfc,
        0x3e, 0x73, 0xd1, 0x6f, 0x00, 0x09, 0xd7, 0x19, 0xd4, 0x1b, 0x78, 0xb1, 0xc5, 0x0d, 0x3b,
        0x4c, 0x82,
    ];
    const BIZ_ID: [u8; 32] = [
        0x4a, 0x4f, 0xb7, 0xff, 0x47, 0x1d, 0xd5, 0xee, 0xc1, 0x2e, 0xde, 0x27, 0x7e, 0xea, 0x16,
        0xdc, 0x35, 0x8f, 0xef, 0x5a, 0x22, 0x54, 0x83, 0xfd, 0xee, 0x94, 0x3d, 0x54, 0xf0, 0x75,
        0xc0, 0xc8,
    ];

    // Test create frozen object
    #[rstest]
    fn test_frozen_object(runtime: RuntimeSandbox) {
        let call_data = createFrozenCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new(()).abi_encode();
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

        // Read initial value (should be 101)
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Increment
        let call_data = incrementValueCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(102, return_data);
        assert_eq!(0, result);

        // Set value to 42
        let call_data = setValueCall::new((42,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(42, return_data);
        assert_eq!(0, result);

        // Increment
        let call_data = incrementValueCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(43, return_data);
        assert_eq!(0, result);

        // Change the signer
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

        // Set value to 111
        let call_data = setValueCall::new((111,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Assert that the value is set
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(111, return_data);
        assert_eq!(0, result);

        // Change the signer
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000acacacac").0.0);

        // Increment
        let call_data = incrementValueCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Assert that the value did not change
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(112, return_data);
        assert_eq!(0, result);

        // Change the msg sender
        runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);

        // Set value to 1111 with a sender that is not the owner
        let call_data = setValueCall::new((1111,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = readValueCall::new(()).abi_encode();
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

        // Read initial value (should be 101)
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Increment
        let call_data = incrementValueCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(102, return_data);
        assert_eq!(0, result);

        // Set value to 42
        let call_data = setValueCall::new((42,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(42, return_data);
        assert_eq!(0, result);

        // Increment
        let call_data = incrementValueCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(43, return_data);
        assert_eq!(0, result);

        // Change the msg sender
        // Should still work since the signer is the owner
        runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);

        // Set value to 111 with a sender that is not the owner
        let call_data = setValueCall::new((111,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Assert that the value was changes correctly
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(111, return_data);
        assert_eq!(0, result);

        // Delete object
        let call_data = deleteObjCall::new(()).abi_encode();
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

        // Compute the object slot using the owner and the object id
        let owner = runtime.get_tx_origin();
        let object_slot = derive_object_slot(&owner, &FOO_ID);

        // Read the storage on the original slot before the freeze
        let value_before_share = runtime.get_storage_at_slot(object_slot.0);

        // Share the object. Only possible if the object is owned by the signer!
        let call_data = shareObjCall::new(()).abi_encode();
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
        let shared_slot = derive_object_slot(&SHARED, &FOO_ID);

        // Read the storage on the shared slot after the share
        // Should be the same as the original slot before the share
        let shared_value = runtime.get_storage_at_slot(shared_slot.0);
        assert_eq!(
            value_before_share, shared_value,
            "Expected storage value to be the same"
        );

        // Read value
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Change the signer and read again
        // Should still work since the object is shared
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
        let call_data = readValueCall::new(()).abi_encode();
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

        // Compute the object slot using the owner and the object id
        let owner = runtime.get_tx_origin();
        let object_slot = derive_object_slot(&owner, &FOO_ID);

        // Read the storage on the original slot before the freeze
        let value_before_freeze = runtime.get_storage_at_slot(object_slot.0);

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new(()).abi_encode();
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
        let frozen_slot = derive_object_slot(&FROZEN, &FOO_ID);

        // Read the storage on the frozen slot after the freeze
        let frozen_value = runtime.get_storage_at_slot(frozen_slot.0);
        assert_eq!(
            value_before_freeze, frozen_value,
            "Expected storage value to be the same"
        );

        // Read value
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Change the signer and read again
        // Should still work since the object is frozen
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Change the msg sender and read again
        // Should still work since the object is frozen
        runtime.set_msg_sender(address!("0x00000000000000000000000000000000abababab").0.0);
        let call_data = readValueCall::new(()).abi_encode();
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

        // Read initial value (should be 101)
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // change the signer
        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

        // This should hit an unreachable due to the signer differing from the owner!
        let call_data = readValueCall::new(()).abi_encode();
        runtime.call_entrypoint(call_data).unwrap();
    }

    // Tests the freeze of an object that is not owned by the signer.
    #[rstest]
    #[should_panic(expected = "unreachable")]
    fn test_freeze_not_owned_object(runtime: RuntimeSandbox) {
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        runtime.set_tx_origin(address!("0x00000000000000000000000000000000abababab").0.0);

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new(()).abi_encode();
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

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new(()).abi_encode();
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

        // Freeze the object. Only possible if the object is owned by the signer!
        let call_data = freezeObjCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        if share {
            // Try to share the object.
            let call_data = shareObjCall::new(()).abi_encode();
            runtime.call_entrypoint(call_data).unwrap();
        } else {
            // Try to transfer the object.
            let call_data = transferObjCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
            runtime.call_entrypoint(call_data).unwrap();
        }
    }

    #[rstest]
    #[should_panic(expected = "unreachable")]
    fn test_delete_frozen_object(runtime: RuntimeSandbox) {
        let call_data = createFrozenCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read value before delete
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Try to delete the object
        let call_data = deleteObjCall::new(()).abi_encode();
        runtime.call_entrypoint(call_data).unwrap();
    }

    // Test delete owned object
    #[rstest]
    fn test_delete_owned_object(runtime: RuntimeSandbox) {
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        // Read value before delete
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Delete the object
        let call_data = deleteObjCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    // Test delete owned object
    #[rstest]
    fn test_delete_shared_object(runtime: RuntimeSandbox) {
        let call_data = createSharedCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        // Read value before delete
        let call_data = readValueCall::new(()).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        // Delete the object
        let call_data = deleteObjCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_get_foo(runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Set value to 111 with a sender that is not the owner
        let call_data = getFooCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Foo::abi_encode(&Foo {
            id: NamedId {
                id: ID {
                    bytes: alloy_primitives::FixedBytes(FOO_ID),
                },
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

        let call_data = getBarCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Bar::abi_encode(&Bar {
            id: NamedId {
                id: ID {
                    bytes: alloy_primitives::FixedBytes(BAR_ID),
                },
            },
            a: 101,
            c: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let call_data = deleteBarCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_delete_baz(runtime: RuntimeSandbox, #[case] share: bool) {
        let call_data = createBazCall::new((SIGNER_ADDRESS.into(), share)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        let call_data = getBazCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Baz::abi_encode(&Baz {
            id: NamedId {
                id: ID {
                    bytes: alloy_primitives::FixedBytes(BAZ_ID),
                },
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

        let call_data = deleteBazCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_delete_bez(runtime: RuntimeSandbox) {
        let call_data = createBezCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        let call_data = getBezCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Bez::abi_encode(&Bez {
            id: NamedId {
                id: ID {
                    bytes: alloy_primitives::FixedBytes(BEZ_ID),
                },
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

        let call_data = deleteBezCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_delete_biz(runtime: RuntimeSandbox) {
        let call_data = createBizCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        let call_data = getBizCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Biz::abi_encode(&Biz {
            id: NamedId {
                id: ID {
                    bytes: alloy_primitives::FixedBytes(BIZ_ID),
                },
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

        let call_data = deleteBizCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }
}

mod storage_transfer {
    use alloy_primitives::{FixedBytes, address};
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

        struct Var {
            UID id;
            Bar a;
        }

        struct Vaz {
            UID id;
            uint32 a;
            Bar b;
            uint64 c;
            Bar d;
        }

        struct EpicVar {
            UID id;
            uint32 a;
            Bar b;
            uint64 c;
            Bar[] d;
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
        // Structs with wrapped objects
        function createVar(address recipient) public view;
        function createVarShared() public view;
        function getVar(bytes32 id) public view returns (Var);
        function shareVar(bytes32 id) public view;
        function freezeVar(bytes32 id) public view;
        function deleteVar(bytes32 id) public view;
        function deleteVarAndTransferBar(bytes32 id) public view;
        function createVaz() public view;
        function getVaz(bytes32 id) public view returns (Vaz);
        function deleteVaz(bytes32 id) public view;
        function createEpicVar() public view;
        function getEpicVar(bytes32 id) public view returns (EpicVar);
        function deleteEpicVar(bytes32 id) public view;
    );

    const SHARED: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1];
    const FROZEN: [u8; 20] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
    const COUNTER_KEY: [u8; 32] = [
        88, 181, 235, 71, 20, 200, 162, 193, 179, 99, 195, 177, 236, 158, 218, 42, 168, 26, 11, 70,
        66, 173, 6, 207, 222, 175, 248, 56, 236, 49, 87, 253,
    ];

    // Test create frozen object
    #[rstest]
    fn test_frozen_object(runtime: RuntimeSandbox) {
        let call_data = createFrozenCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.obtain_uid();

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
        let object_id = runtime.obtain_uid();

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
        let object_id = runtime.obtain_uid();

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
        let object_id = runtime.obtain_uid();

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
        let object_id = runtime.obtain_uid();

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
        let object_id = runtime.obtain_uid();

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
        let object_id = runtime.obtain_uid();

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
        let object_id = runtime.obtain_uid();

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
        let object_id = runtime.obtain_uid();

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

        let object_id = runtime.obtain_uid();

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

        let object_id = runtime.obtain_uid();

        // Read value before delete
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        // Delete the object
        let call_data = deleteObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_delete_shared_object(runtime: RuntimeSandbox) {
        let call_data = createSharedCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = runtime.obtain_uid();

        // Read value before delete
        let call_data = readValueCall::new((object_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readValueCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(101, return_data);
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        // Delete the object
        let call_data = deleteObjCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_get_foo(runtime: RuntimeSandbox) {
        // Create a new counter
        let call_data = createOwnedCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let object_id = runtime.obtain_uid();

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

        let object_id = runtime.obtain_uid();

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
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_delete_baz(runtime: RuntimeSandbox, #[case] share: bool) {
        let call_data = createBazCall::new((SIGNER_ADDRESS.into(), share)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        let object_id = runtime.obtain_uid();

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
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_delete_bez(runtime: RuntimeSandbox) {
        let call_data = createBezCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        let object_id = runtime.obtain_uid();

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

        let object_id = runtime.obtain_uid();

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

        let object_1_id = runtime.obtain_uid();

        let call_data = createSharedCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_2_id = runtime.obtain_uid();

        let storage_before_delete = runtime.get_storage();

        let call_data = deleteObj2Call::new((object_1_id, object_2_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_delete_owned_var(runtime: RuntimeSandbox) {
        let call_data = createVarCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = runtime.obtain_uid();

        let call_data = getVarCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Var::abi_encode(&Var {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 42,
                c: vec![1, 2, 3],
            },
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let storage_before_delete = runtime.get_storage();

        let call_data = deleteVarCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    // First we test deleting a Var shared upon creation
    // Then we test deleting a Var owned upon creation and later shared
    #[rstest]
    fn test_delete_shared_var(runtime: RuntimeSandbox) {
        let call_data = createVarSharedCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = runtime.obtain_uid();

        let call_data = getVarCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Var::abi_encode(&Var {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 42,
                c: vec![1, 2, 3],
            },
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let storage_before_delete = runtime.get_storage();

        let call_data = deleteVarCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);

        // Create owned var and share it, then delete it
        let call_data = createVarCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = FixedBytes::<32>::from_slice(
            &hex::decode("b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255")
                .unwrap(),
        );

        let call_data = getVarCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Var::abi_encode(&Var {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "1f0c5f0153ea5a939636c6a5f255f2fb613b03bef89fb34529e246fe1697a741",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 42,
                c: vec![1, 2, 3],
            },
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let call_data = shareVarCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = getVarCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Var::abi_encode(&Var {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "1f0c5f0153ea5a939636c6a5f255f2fb613b03bef89fb34529e246fe1697a741",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 42,
                c: vec![1, 2, 3],
            },
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let storage_before_delete = runtime.get_storage();

        let call_data = deleteVarCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_freeze_owned_var(runtime: RuntimeSandbox) {
        // Create owned var and freeze it
        let call_data = createVarCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = FixedBytes::<32>::from_slice(
            &hex::decode("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")
                .unwrap(),
        );

        let call_data = freezeVarCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Check if the 2 slots corresponding to the Var struct under the SIGNER_ADDRESS key are empty after the freeze
        let var_uid_slot_bytes: [u8; 32] = [
            93, 247, 165, 32, 139, 166, 195, 236, 47, 167, 72, 117, 174, 153, 62, 53, 76, 142, 238,
            122, 118, 205, 148, 75, 134, 218, 76, 250, 55, 149, 13, 24,
        ];
        let bar_uid_slot_bytes: [u8; 32] = [
            93, 247, 165, 32, 139, 166, 195, 236, 47, 167, 72, 117, 174, 153, 62, 53, 76, 142, 238,
            122, 118, 205, 148, 75, 134, 218, 76, 250, 55, 149, 13, 25,
        ];

        // Check if the slots exist and are zero
        let value = runtime.get_storage_at_slot(var_uid_slot_bytes);
        assert_eq!(value, [0u8; 32], "Var UID slot should be zero");

        let value = runtime.get_storage_at_slot(bar_uid_slot_bytes);
        assert_eq!(value, [0u8; 32], "Bar UID slot should be zero");

        let call_data = getVarCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Var::abi_encode(&Var {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 42,
                c: vec![1, 2, 3],
            },
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);
    }

    #[rstest]
    // Delete var and share bar
    // Check that all original slots are empty: Var is delete and Bar is moved
    // After that, try getting bar and then deleting it
    fn test_delete_var_and_transfer_bar(runtime: RuntimeSandbox) {
        let call_data = createVarCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = FixedBytes::<32>::from_slice(
            &hex::decode("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")
                .unwrap(),
        );

        let storage_before_delete = runtime.get_storage();

        // Delete var and share bar
        let call_data = deleteVarAndTransferBarCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete: std::collections::HashMap<[u8; 32], [u8; 32]> =
            runtime.get_storage();
        for key in storage_before_delete.keys() {
            if *key != COUNTER_KEY {
                let value = storage_after_delete.get(key).unwrap_or(&[0u8; 32]);
                assert_eq!(
                    *value, [0u8; 32],
                    "Unexpected non-zero value at key: {:?}",
                    key
                );
            }
        }

        // Bar id
        let object_id = FixedBytes::<32>::from_slice(
            &hex::decode("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989")
                .unwrap(),
        );

        let call_data = getBarCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        let expected_result = Bar::abi_encode(&Bar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 42,
            c: vec![1, 2, 3],
        });
        assert_eq!(result_data, expected_result);

        let storage_before_delete = runtime.get_storage();

        let call_data = deleteBarCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_delete_vaz(runtime: RuntimeSandbox) {
        let call_data = createVazCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = runtime.obtain_uid();

        let call_data = getVazCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        let expected_result = Vaz::abi_encode(&Vaz {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 101,
            b: Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 42,
                c: vec![1, 2, 3],
            },
            c: 102,
            d: Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 43,
                c: vec![4, 5, 6],
            },
        });
        assert_eq!(0, result);
        assert_eq!(result_data, expected_result);

        let storage_before_delete = runtime.get_storage();

        let call_data = deleteVazCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_delete_epic_var(runtime: RuntimeSandbox) {
        let call_data = createEpicVarCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let object_id = runtime.obtain_uid();

        let call_data = getEpicVarCall::new((object_id,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        let expected_result = EpicVar::abi_encode(&EpicVar {
            id: UID {
                id: ID {
                    bytes: U256::from_str_radix(
                        "7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de",
                        16,
                    )
                    .unwrap()
                    .into(),
                },
            },
            a: 101,
            b: Bar {
                id: UID {
                    id: ID {
                        bytes: U256::from_str_radix(
                            "bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989",
                            16,
                        )
                        .unwrap()
                        .into(),
                    },
                },
                a: 41,
                c: vec![1, 2, 3],
            },
            c: 102,
            d: vec![
                Bar {
                    id: UID {
                        id: ID {
                            bytes: U256::from_str_radix(
                                "b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255",
                                16,
                            )
                            .unwrap()
                            .into(),
                        },
                    },
                    a: 42,
                    c: vec![42, 43],
                },
                Bar {
                    id: UID {
                        id: ID {
                            bytes: U256::from_str_radix(
                                "1f0c5f0153ea5a939636c6a5f255f2fb613b03bef89fb34529e246fe1697a741",
                                16,
                            )
                            .unwrap()
                            .into(),
                        },
                    },
                    a: 43,
                    c: vec![44, 45, 46],
                },
            ],
        });
        assert_eq!(result_data, expected_result);

        let storage_before_delete = runtime.get_storage();

        let call_data = deleteEpicVarCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
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
           bytes32 bytes;
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
            uint256 a,
            uint128 b,
            uint64 c,
            uint32 d,
            uint16 e,
            uint8 f,
            address g
        ) public view;
        function readStaticFields(uint256 id) public view returns (StaticFields);

        function saveStaticFields2(
            uint8 a,
            address b,
            uint64 c,
            uint16 d,
            uint8 e
        ) public view;
        function readStaticFields2(uint256 id) public view returns (StaticFields2);

        function saveStaticFields3(
            uint8 a,
            address b,
            uint64 c,
            address d
        ) public view;
        function readStaticFields3(uint256 id) public view returns (StaticFields3);

        function saveStaticNestedStruct(
            uint64 a,
            bool b,
            uint64 d,
            address e,
            uint128 f,
            uint32 g
        ) public view;
        function readStaticNestedStruct(uint256 id) public view returns (StaticNestedStruct);

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
            uint32 a,
            bool b,
            uint64[] c,
            uint128[] d,
            uint64 e,
            uint128 f,
            uint256 g,
        ) public view;
        function readDynamicStruct(uint256 id) public view returns (DynamicStruct);

        function saveDynamicStruct2(
            bool[] a,
            uint8[] b,
            uint16[] c,
            uint32[] d,
            uint64[] e,
            uint128[] f,
            uint256[] g,
            address[] h,
        ) public view;
        function readDynamicStruct2(uint256 id) public view returns (DynamicStruct2);

        function saveDynamicStruct3(
            uint8[][] a,
            uint32[][] b,
            uint64[][] c,
            uint128[][] d,
        ) public view;
        function readDynamicStruct3(uint256 id) public view returns (DynamicStruct3);

        function saveDynamicStruct4(
            uint32[] x,
            uint64 y,
            uint128 z,
            address w,
        ) public view;
        function readDynamicStruct4(uint256 id) public view returns (DynamicStruct4);

        function saveDynamicStruct5(
            uint32 x,
            uint64 y,
            uint128 z,
            address w,
        ) public view;
        function readDynamicStruct5(uint256 id) public view returns (DynamicStruct5);

        function saveGenericStruct32(
            uint32 x,
        ) public view;
        function readGenericStruct32(uint256 id) public view returns (GenericStruct32);

        //// Wrapped objects ////
        struct Foo {
            UID id;
            uint64 a;
            Bar b;
            uint32 c;
        }

        struct Bar {
            UID id;
            uint64 a;
        }
        function saveFoo () public view;
        function readFoo(uint256 id) public view returns (Foo);

        struct MegaFoo {
            UID id;
            uint64 a;
            Foo b;
            uint32 c;
        }
        function saveMegaFoo() public view;
        function readMegaFoo(uint256 id) public view returns (MegaFoo);

        struct Var {
            UID id;
            Bar a;
            Foo b;
            Bar[] c;
        }
        function saveVar() public view;
        function readVar(uint256 id) public view returns (Var);

        struct GenericWrapper32 {
            UID id;
            uint32 a;
            GenericStruct32 b;
            uint32 c;
        }
        function saveGenericWrapper32() public view;
        function readGenericWrapper32(uint256 id) public view returns (GenericWrapper32);
    );

    #[rstest]
    #[case(saveStaticFieldsCall::new((
        U256::from_str_radix("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 16).unwrap(),
        0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb,
        0xcccccccccccccccc,
        0xdddddddd,
        0xeeee,
        0xff,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
    )), vec![
        U256::from_str_radix("000000000000000000000000000000000000000000000000befe6eaf1b07b760", 16).unwrap().to_be_bytes(),
        [0xaa; 32],
        U256::from_str_radix("ffeeeeddddddddccccccccccccccccbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
    ],
        readStaticFieldsCall::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        StaticFields {
            id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
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
        U256::from(1),
        2,
        3,
        4,
        5,
        6,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
    )), vec![
        U256::from_str_radix("000000000000000000000000000000000000000000000000befe6eaf1b07b760", 16).unwrap().to_be_bytes(),
        U256::from(1).to_be_bytes(),
        U256::from_str_radix("06000500000004000000000000000300000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
    ],
        readStaticFieldsCall::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        StaticFields {
            id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
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
        0xff,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        0xcccccccccccccccc,
        0xeeee,
        0xff,
    )), vec![
        U256::from_str_radix("000000cafecafecafecafecafecafecafecafecafecafeff8302f0af284e5ac4", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000ffeeeecccccccccccccccc", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields2Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        StaticFields2 {
            id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
            a: 0xff,
            b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
            c: 0xcccccccccccccccc,
            d: 0xeeee,
            e: 0xff,
        }
    )]
    #[case(saveStaticFields2Call::new((
        1,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        2,
        3,
        4,
    )), vec![
        U256::from_str_radix("000000cafecafecafecafecafecafecafecafecafecafe018302f0af284e5ac4", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000400030000000000000002", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields2Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        StaticFields2 {
            id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
            a: 1,
            b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
            c: 2,
            d: 3,
            e: 4,
        }
    )]
    #[case(saveStaticFields3Call::new((
        1,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        2,
        address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
    )), vec![
        U256::from_str_radix("000000cafecafecafecafecafecafecafecafecafecafe01b3f6054d24105600", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000beefbeefbeefbeefbeefbeefbeefbeefbeefbeef0000000000000002", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields3Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        StaticFields3 {
           id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
           a: 1,
           b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
           c: 2,
           d: address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
        }
    )]
    #[case(saveStaticFields3Call::new((
        0xff,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        0xcccccccccccccccc,
        address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
    )), vec![
        U256::from_str_radix("000000cafecafecafecafecafecafecafecafecafecafeffb3f6054d24105600", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000beefbeefbeefbeefbeefbeefbeefbeefbeefbeefcccccccccccccccc", 16).unwrap().to_be_bytes(),
    ],
        readStaticFields3Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        StaticFields3 {
            id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
            a: 0xff,
            b: address!("0xcafecafecafecafecafecafecafecafecafecafe"),
            c: 0xcccccccccccccccc,
            d: address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
        }
    )]
    #[case(saveStaticNestedStructCall::new((
        1,
        true,
        2,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        3,
        4
    )), vec![
        U256::from_str_radix("000000000000000000000000000002010000000000000001ab422efcbfadd563", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000400000000000000000000000000000003", 16).unwrap().to_be_bytes(),
    ],
        readStaticNestedStructCall::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        StaticNestedStruct {
           id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
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
        0xaaaaaaaaaaaaaaaa,
        true,
        0xbbbbbbbbbbbbbbbb,
        address!("0xcafecafecafecafecafecafecafecafecafecafe"),
        0xcccccccccccccccccccccccccccccccc,
        0xdddddddd,
    )), vec![
        U256::from_str_radix("00000000000000bbbbbbbbbbbbbbbb01aaaaaaaaaaaaaaaaab422efcbfadd563", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000cafecafecafecafecafecafecafecafecafecafe", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000ddddddddcccccccccccccccccccccccccccccccc", 16).unwrap().to_be_bytes(),
    ],
        readStaticNestedStructCall::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        StaticNestedStruct {
           id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
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
        46,
        true,
        vec![2, 3, 4, 5, 6],
        vec![7, 8, 9],
        47,
        48,
        U256::from(49),
    )),
    vec![
        [0x00; 32],
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // 0x01 (vector header)
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // 0x02 (vector header)
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // 0x03 u64 and u128 slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(), // 0x04 u64 and u128 slot

        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(), // vector elements first slot
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf", 16).unwrap().to_be_bytes(), // vector elements second slot

        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // vector elements first slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf7", 16).unwrap().to_be_bytes(), // vector elements second slot

    ],
    vec![
        U256::from_str_radix("00000000000000000000000000000000000000010000002e83f4d1b6a8351bb2", 16).unwrap().to_be_bytes(), // type hash + u32 + bool
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000005", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("000000000000000000000000000000000000000000000030000000000000002f", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000031", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000800000000000000000000000000000007", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000009", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("0000000000000005000000000000000400000000000000030000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000006", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStructCall::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        DynamicStruct {
           id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
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
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000004", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("405787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5acf", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000001ffffffff83f4d1b6a8351bb2", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000000", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000000000000000000000000000000000000000030ffffffffffffffff", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000031", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000800000000000000000000000000000007", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000009", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStructCall::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        DynamicStruct {
           id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
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
        U256::from_str_radix("000000000000000000000000000000000000000000000000c459c3743aeb04b6", 16).unwrap().to_be_bytes(),
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
        readDynamicStruct2Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        DynamicStruct2 {
        id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
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
        U256::from_str_radix("000000000000000000000000000000000000000000000000ebc75c981e6b1aa3", 16).unwrap().to_be_bytes(),
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
        readDynamicStruct3Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        DynamicStruct3 {
           id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
           a: vec![vec![1, 2, 3], vec![4, 5]],
           b: vec![vec![6, 7], vec![8], vec![9, 10]],
           c: vec![vec![11, 12, 13, 14], vec![], vec![15, 16]],
           d: vec![vec![17, 18, 19]],
        }
    )]
    #[case(saveDynamicStruct4Call::new((
        vec![1, 2, 3],
        47,
        123,
        address!("1111111111111111111111111111111111111111"),
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
        U256::from_str_radix("0000000000000000000000000000000000000000000000007509f0567373e5f0", 16).unwrap().to_be_bytes(),
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
        readDynamicStruct4Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        DynamicStruct4 {
        id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
           a: vec![DynamicNestedStructChild { a: vec![1, 2, 3], b: 123 }, DynamicNestedStructChild { a: vec![1, 2, 3], b: 124 }],
           b: vec![StaticNestedStructChild { d: 47, e: address!("0x1111111111111111111111111111111111111111") }, StaticNestedStructChild { d: 48, e: address!("0x1111111111111111111111111111111111111111") }, StaticNestedStructChild { d: 49, e: address!("0x1111111111111111111111111111111111111111") }],
        }
    )]
    #[case(saveDynamicStruct5Call::new((
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
        U256::from_str_radix("0000000000000000000000000000000000000000000000003a71241b10e629e0", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
    ],
        readDynamicStruct5Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        DynamicStruct5 {
        id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
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
        1,
    )),
    vec![
        [0x00; 32], // 0x0
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // uint32[] header slot
        U256::from_str_radix("b10e2d527612073b26eecdfd717e6a320cf44b4afac2b0732d9fcbe2b7fa0cf6", 16).unwrap().to_be_bytes(), // uint32[] elements slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(), // uint32 b
    ],
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000007767397bdbd83f17", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(), // Header slot
        U256::from_str_radix("0000000000000000000000000000000000000000000000030000000200000001", 16).unwrap().to_be_bytes(), // First element
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(), // Second element
    ],
        readGenericStruct32Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
        GenericStruct32 {
            id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into() } },
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

    #[rstest]
    #[case(saveFooCall::new(()),
    vec![
        [0x00; 32],
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("b3f87769e0f4505eb27364fe9b31c117ff789e8aa785586680a6c1cb0f592652", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("0000000000000000000000000000000000000000000000653d02a210b08857c8", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000066", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("00000000000000000000000000000000000000000000002a1afe8c88bc2c2d3f", 16).unwrap().to_be_bytes(),
    ],
        readFooCall::new((U256::from_le_bytes(hex!("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989")),)),
        Foo {
            id: UID { id: ID { bytes: U256::from_str_radix("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989", 16).unwrap().into()  } },
            a: 101,
            b: Bar {
                id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into()  } },
                a: 42,
            },
            c: 102,
        }
    )]
    #[case(saveMegaFooCall::new(()),
    vec![
        // MegaFoo
        [0x00; 32],
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        // Foo
        U256::from_str_radix("3560195e435d1f629e64ba21b204d3190c17b5cf38d1e100e4939dff8e98638d", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("3560195e435d1f629e64ba21b204d3190c17b5cf38d1e100e4939dff8e98638e", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("3560195e435d1f629e64ba21b204d3190c17b5cf38d1e100e4939dff8e98638f", 16).unwrap().to_be_bytes(),
        //Bar
        U256::from_str_radix("b3f87769e0f4505eb27364fe9b31c117ff789e8aa785586680a6c1cb0f592652", 16).unwrap().to_be_bytes(),
    ],
    vec![
        // MegaFoo
        U256::from_str_radix("00000000000000000000000000000000000000000000004d68e61705cbfc7a75", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000058", 16).unwrap().to_be_bytes(),
        // Foo
        U256::from_str_radix("0000000000000000000000000000000000000000000000653d02a210b08857c8", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000066", 16).unwrap().to_be_bytes(),
        // Bar
        U256::from_str_radix("00000000000000000000000000000000000000000000002a1afe8c88bc2c2d3f", 16).unwrap().to_be_bytes(),

    ],
        readMegaFooCall::new((U256::from_le_bytes(hex!("b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255")),)),
    MegaFoo {
            id: UID { id: ID { bytes: U256::from_str_radix("b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255", 16).unwrap().into()  } },
            a: 77,
            b: Foo {
                id: UID { id: ID { bytes: U256::from_str_radix("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989", 16).unwrap().into()  } },
                a: 101,
                b: Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into()  } },
                    a: 42,
                },
                c: 102,
            },
            c: 88,
        }
    )]
    #[case(saveVarCall::new(()),
    vec![
        // Var
        [0x00; 32],
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        //Bar
        U256::from_str_radix("918d490f3f5a5af006896b3a37d65a9b496b1db689b87334200c90ab4023c178", 16).unwrap().to_be_bytes(),
        // Foo
        U256::from_str_radix("322a33ca5945c1c34a8261d3e947c675478f7d28fef6f722a010b01be97bd034", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("322a33ca5945c1c34a8261d3e947c675478f7d28fef6f722a010b01be97bd035", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("322a33ca5945c1c34a8261d3e947c675478f7d28fef6f722a010b01be97bd036", 16).unwrap().to_be_bytes(),
        //Bar in Foo
        U256::from_str_radix("751256a7a9ce5df532239c40b31940a9a4fe03b965b20192b8f02653745a9369", 16).unwrap().to_be_bytes(),
        // Bar vector
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85b", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85c", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("c2575a0e9e593c00f959f8c92f12db2869c3395a3b0502d05e2516446f71f85d", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("ae7e0571fa79d756545e084ec0d0c624cdbe4a05e898a1cf0e32a0821c5a6911", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("26a28b6df88122dd5c0e1993dcda278372d63dbd74d0b08e3d76c6012a7da7f8", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("e41226abe04fb72908c78250de568834065f6d35bc75081a3368d6a693296f5a", 16).unwrap().to_be_bytes(),
    ],
    vec![
        // Var
        U256::from_str_radix("000000000000000000000000000000000000000000000000a05005766d1c7798", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("d929b3310243fae82a39e83032462fceb274b042b98732db8c6e9fbeab70c3c9", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        // Bar
        U256::from_str_radix("00000000000000000000000000000000000000000000002a1afe8c88bc2c2d3f", 16).unwrap().to_be_bytes(),
        // Foo
        U256::from_str_radix("0000000000000000000000000000000000000000000000653d02a210b08857c8", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000066", 16).unwrap().to_be_bytes(),
        // Bar in Foo
        U256::from_str_radix("0000000000000000000000000000000000000000000000291afe8c88bc2c2d3f", 16).unwrap().to_be_bytes(),
        // Bar vector
        U256::from_str_radix("b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("1f0c5f0153ea5a939636c6a5f255f2fb613b03bef89fb34529e246fe1697a741", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("60b770a33dfbcb5aaea4306257d155502df85b76449b216c476fcfcd437c152e", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("00000000000000000000000000000000000000000000002b1afe8c88bc2c2d3f", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("00000000000000000000000000000000000000000000002c1afe8c88bc2c2d3f", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("00000000000000000000000000000000000000000000002d1afe8c88bc2c2d3f", 16).unwrap().to_be_bytes(),

    ],
    readVarCall::new((U256::from_le_bytes(hex!("d51bb5edad7d1535fb0a47b2d03d08c0fe02560a3de80e55815fedb1ce1be09b")),)),
            Var {
            id: UID { id: ID { bytes: U256::from_str_radix("d51bb5edad7d1535fb0a47b2d03d08c0fe02560a3de80e55815fedb1ce1be09b", 16).unwrap().into()  } },
            a: Bar {
                id: UID { id: ID { bytes: U256::from_str_radix("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989", 16).unwrap().into()  } },
                a: 42,
            },
            b: Foo {
                id: UID { id: ID { bytes: U256::from_str_radix("d929b3310243fae82a39e83032462fceb274b042b98732db8c6e9fbeab70c3c9", 16).unwrap().into()  } },
                a: 101,
                b: Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into()  } },
                    a: 41,
                },
                c: 102,
            },
            c: vec![
                Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("b067f9efb12a40ca24b641163e267b637301b8d1b528996becf893e3bee77255", 16).unwrap().into()  } },
                    a: 43,
                },
                Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("1f0c5f0153ea5a939636c6a5f255f2fb613b03bef89fb34529e246fe1697a741", 16).unwrap().into()  } },
                    a: 44,
                },
                Bar {
                    id: UID { id: ID { bytes: U256::from_str_radix("60b770a33dfbcb5aaea4306257d155502df85b76449b216c476fcfcd437c152e", 16).unwrap().into()  } },
                    a: 45,
                }
            ],
        }
    )]
    #[case(saveGenericWrapper32Call::new(()),
    vec![
        [0x00; 32],
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000001", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000002", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("398fdf7528e5068055009aa3b7c48e06f0127b5d8c57be483a07b5cd9100322e", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("398fdf7528e5068055009aa3b7c48e06f0127b5d8c57be483a07b5cd9100322f", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("398fdf7528e5068055009aa3b7c48e06f0127b5d8c57be483a07b5cd91003230", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("580978fee9799ff96dcfcd540232ed2f7cd5bade678a1ccae6650e39d39559dd", 16).unwrap().to_be_bytes(),
    ],
    vec![
        U256::from_str_radix("000000000000000000000000000000000000000000000065d0ae4436393e9304", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000066", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("0000000000000000000000000000000000000000000000007767397bdbd83f17", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("0000000000000000000000000000000000000000000000000000000000000003", 16).unwrap().to_be_bytes(),
        U256::from_str_radix("00000000000000000000000000000000000000000000000000000000000004d2", 16).unwrap().to_be_bytes(),

        U256::from_str_radix("000000000000000000000000000000000000000000000063000000580000004d", 16).unwrap().to_be_bytes(),
    ],
    readGenericWrapper32Call::new((U256::from_le_bytes(hex!("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de")),)),
    GenericWrapper32 {
            id: UID { id: ID { bytes: U256::from_str_radix("7ce17a84c7895f542411eb103f4973681391b4fb07cd0d099a6b2e70b25fa5de", 16).unwrap().into()  } },
            a: 101,
            b: GenericStruct32 {
                id: UID { id: ID { bytes: U256::from_str_radix("bde695b08375ca803d84b5f0699ca6dfd57eb08efbecbf4c397270aae24b9989", 16).unwrap().into()  } },
                a: vec![77, 88, 99],
                b: 1234,
            },
            c: 102,
        }
    )]
    fn test_wrapped_objects<T: SolCall, U: SolCall, V: SolValue>(
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

mod trusted_swap {
    use alloy_sol_types::SolValue;
    use alloy_sol_types::{SolCall, sol};

    use super::*;

    // NOTE: we can't use this fixture as #[once] because in order to catch events, we use an mpsc
    // channel. If we use this as #[once], there's a possibility this runtime is used in more than one
    // thread. If that happens, messages from test A can be received by test B.
    // Using once instance per thread assures this won't happen.
    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "trusted_swap";
        const SOURCE_PATH: &str = "tests/storage/trusted_swap.move";

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

        struct Object {
            UID id;
            uint8 scarcity;
            uint8 style;
        }

        struct SwapRequest {
            UID id;
            address owner;
            Object object;
            uint64 fee;
        }

        function createObject(uint8 scarcity, uint8 style) public;
        function readObject(bytes32 id) public view returns (Object);
        function requestSwap(bytes32 id, address service, uint64 fee) public;
        function executeSwap(bytes32 id1, bytes32 id2) public returns (uint64);
    );

    const OWNER_A: [u8; 20] = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];
    const OWNER_B: [u8; 20] = [2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];
    const SERVICE: [u8; 20] = [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3];

    #[rstest]
    fn test_successful_swap(runtime: RuntimeSandbox) {
        ////// First owner creates an object //////
        runtime.set_msg_sender(OWNER_A);
        runtime.set_tx_origin(OWNER_A);
        let fee_a = 1000;

        let call_data = createObjectCall::new((7, 2)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let obj_a_id = runtime.obtain_uid();

        let obj_a_slot = derive_object_slot(&OWNER_A, &obj_a_id.0);

        let call_data = readObjectCall::new((obj_a_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readObjectCall::abi_decode_returns(&return_data).unwrap();
        let obj_a_expected = Object::abi_encode(&Object {
            id: UID {
                id: ID { bytes: obj_a_id },
            },
            scarcity: 7,
            style: 2,
        });
        assert_eq!(Object::abi_encode(&return_data), obj_a_expected);
        assert_eq!(0, result);

        let call_data = requestSwapCall::new((obj_a_id, SERVICE.into(), fee_a)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the swap request id emmited from the contract's events
        let swap_request_a_id = runtime.obtain_uid();
        println!("Swap Request A ID: {:#x}", swap_request_a_id);

        // Assert that the slot is empty
        assert_eq!(
            runtime.get_storage_at_slot(obj_a_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );

        ////// Second owner requests a swap //////
        runtime.set_msg_sender(OWNER_B);
        runtime.set_tx_origin(OWNER_B);
        let fee_b = 1250;

        let call_data = createObjectCall::new((7, 3)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let obj_b_id = runtime.obtain_uid();

        let obj_b_slot = derive_object_slot(&OWNER_B, &obj_b_id.0);

        let call_data = readObjectCall::new((obj_b_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readObjectCall::abi_decode_returns(&return_data).unwrap();
        let obj_b_expected = Object::abi_encode(&Object {
            id: UID {
                id: ID { bytes: obj_b_id },
            },
            scarcity: 7,
            style: 3,
        });
        assert_eq!(Object::abi_encode(&return_data), obj_b_expected);
        assert_eq!(0, result);

        let call_data = requestSwapCall::new((obj_b_id, SERVICE.into(), fee_b)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let swap_request_b_id = runtime.obtain_uid();

        // Assert that the slot is empty
        assert_eq!(
            runtime.get_storage_at_slot(obj_b_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );

        ////// Execute the swap //////
        runtime.set_msg_sender(SERVICE);
        runtime.set_tx_origin(SERVICE);

        let storage_before_delete = runtime.get_storage();

        let call_data = executeSwapCall::new((swap_request_a_id, swap_request_b_id)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = executeSwapCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(fee_a + fee_b, return_data);
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        assert_empty_storage(&storage_before_delete, &storage_after_delete);

        for (key, value) in storage_after_delete.iter() {
            if *key != COUNTER_KEY && value != &[0u8; 32] {
                println!("{:?} \n {:?} \n", key, value);
            }
        }

        ////// Read the objects //////
        // Now owner A should have the object B, and owner B should have the object A.
        runtime.set_msg_sender(OWNER_A);
        runtime.set_tx_origin(OWNER_A);

        let call_data = readObjectCall::new((obj_b_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readObjectCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(Object::abi_encode(&return_data), obj_b_expected);
        assert_eq!(0, result);

        runtime.set_msg_sender(OWNER_B);
        runtime.set_tx_origin(OWNER_B);

        let call_data = readObjectCall::new((obj_a_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readObjectCall::abi_decode_returns(&return_data).unwrap();
        assert_eq!(Object::abi_encode(&return_data), obj_a_expected);
        assert_eq!(0, result);
    }

    #[rstest]
    #[should_panic]
    fn test_swap_too_cheap(runtime: RuntimeSandbox) {
        // Create an object
        runtime.set_msg_sender(OWNER_A);
        runtime.set_tx_origin(OWNER_A);

        let call_data = createObjectCall::new((7, 2)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let obj_a_id = runtime.obtain_uid();

        // Request a swap with a fee too low
        let fee_a = 999;
        let call_data = requestSwapCall::new((obj_a_id, SERVICE.into(), fee_a)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }

    #[rstest]
    #[should_panic]
    fn test_swap_different_scarcity(runtime: RuntimeSandbox) {
        ////// First owner creates an object //////
        runtime.set_msg_sender(OWNER_A);
        runtime.set_tx_origin(OWNER_A);
        let fee_a = 1000;

        let call_data = createObjectCall::new((7, 2)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let obj_a_id = runtime.obtain_uid();

        let call_data = requestSwapCall::new((obj_a_id, SERVICE.into(), fee_a)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the swap request id emmited from the contract's events
        let swap_request_a_id = runtime.obtain_uid();

        ////// Second owner requests a swap //////
        runtime.set_msg_sender(OWNER_B);
        runtime.set_tx_origin(OWNER_B);
        let fee_b = 1250;

        let call_data = createObjectCall::new((8, 3)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let obj_b_id = runtime.obtain_uid();

        let call_data = requestSwapCall::new((obj_b_id, SERVICE.into(), fee_b)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let swap_request_b_id = runtime.obtain_uid();

        ////// Execute the swap //////
        runtime.set_msg_sender(SERVICE);
        runtime.set_tx_origin(SERVICE);

        let call_data = executeSwapCall::new((swap_request_a_id, swap_request_b_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }

    #[rstest]
    #[should_panic]
    fn test_swap_same_style(runtime: RuntimeSandbox) {
        ////// First owner creates an object //////
        runtime.set_msg_sender(OWNER_A);
        runtime.set_tx_origin(OWNER_A);
        let fee_a = 1000;

        let call_data = createObjectCall::new((7, 3)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let obj_a_id = runtime.obtain_uid();

        let call_data = requestSwapCall::new((obj_a_id, SERVICE.into(), fee_a)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the swap request id emmited from the contract's events
        let swap_request_a_id = runtime.obtain_uid();

        ////// Second owner requests a swap //////
        runtime.set_msg_sender(OWNER_B);
        runtime.set_tx_origin(OWNER_B);
        let fee_b = 1250;

        let call_data = createObjectCall::new((7, 3)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the object id emmited from the contract's events
        let obj_b_id = runtime.obtain_uid();

        let call_data = requestSwapCall::new((obj_b_id, SERVICE.into(), fee_b)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let swap_request_b_id = runtime.obtain_uid();

        ////// Execute the swap //////
        runtime.set_msg_sender(SERVICE);
        runtime.set_tx_origin(SERVICE);

        let call_data = executeSwapCall::new((swap_request_a_id, swap_request_b_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
    }
}
mod wrapped_objects {
    use crate::common::runtime_sandbox::constants::MSG_SENDER_ADDRESS;
    use alloy_sol_types::{SolCall, SolValue, sol};

    use super::*;

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "wrapped_objects";
        const SOURCE_PATH: &str = "tests/storage/wrapped_objects.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        let runtime = RuntimeSandbox::new(&mut translated_package);
        runtime.set_tx_origin(MSG_SENDER_ADDRESS);
        runtime
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

        struct Alpha {
            UID id;
            uint64 value;
        }

        struct Beta {
            UID id;
            Alpha a;
        }

        struct Gamma {
            UID id;
            Beta a;
        }

        struct Delta {
            UID id;
            Alpha[] a;
        }

        struct Epsilon {
            UID id;
            Delta[] a;
        }
        struct Zeta {
            UID id;
            Astra b;
        }

        struct Astra {
            Alpha[] a;
        }

        struct Eta {
            UID id;
            Bora b;
        }

        struct Bora {
            uint64[] a;
            uint64[][] b;
        }
        function createAlpha(uint64 value) public view;
        function createBeta() public view;
        function createGamma() public view;
        function createDelta() public view;
        function createEmptyDelta() public view;
        function createEpsilon() public view;
        function createEmptyZeta() public view;
        function createBetaTto(bytes32 a) public view;
        function createGammaTto(bytes32 a) public view;
        function createDeltaTto(bytes32 a, bytes32 b) public view;
        function createEpsilonTto(bytes32 a, bytes32 b) public view;
        function createEta() public view;
        function readAlpha(bytes32 a) public view returns (Alpha);
        function readBeta(bytes32 b) public view returns (Beta);
        function readGamma(bytes32 g) public view returns (Gamma);
        function readDelta(bytes32 d) public view returns (Delta);
        function readEpsilon(bytes32 e) public view returns (Epsilon);
        function readZeta(bytes32 z) public view returns (Zeta);
        function readEta(bytes32 id) public view returns (Eta);
        function deleteAlpha(bytes32 a) public view;
        function deleteBeta(bytes32 b) public view;
        function deleteGamma(bytes32 g) public view;
        function deleteDelta(bytes32 d) public view;
        function deleteZeta(bytes32 z) public view;
        function deleteEpsilon(bytes32 e) public view;
        function transferBeta(bytes32 b, address recipient) public view;
        function transferGamma(bytes32 g, address recipient) public view;
        function transferDelta(bytes32 d, address recipient) public view;
        function transferZeta(bytes32 z, address recipient) public view;
        function rebuildGamma(bytes32 g, address recipient) public view;
        function destructDeltaToBeta(bytes32 d) public view;
        function pushAlphaToDelta(bytes32 d, bytes32 a) public view;
        function popAlphaFromDelta(bytes32 d) public view;
        function destructEpsilon(bytes32 e, bytes32 a) public view;
        function pushAlphaToZeta(bytes32 z, bytes32 a) public view;
        function popAlphaFromZeta(bytes32 z) public view;
        function pushToBora(bytes32 e, uint64 v) public view;
        function popFromBora(bytes32 e) public returns (uint64, uint64[]);
    );

    // In all tests, we use the tto flag to indicate if the creation method should take
    // the object to be wrapped as argument or create it directly.
    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_creating_and_deleting_beta(runtime: RuntimeSandbox, #[case] tto: bool) {
        let (alpha_id, beta_id) = if tto {
            // Create alpha first for TTO method
            let call_data = createAlphaCall::new((102,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let alpha_id = runtime.obtain_uid();

            // Create beta, passing alpha as argument to be wrapped in it
            let call_data = createBetaTtoCall::new((alpha_id,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let beta_id = runtime.obtain_uid();

            (alpha_id, beta_id)
        } else {
            // Create beta directly
            let call_data = createBetaCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            // Get the object ids
            let alpha_id = runtime.obtain_uid();
            let beta_id = runtime.obtain_uid();

            (alpha_id, beta_id)
        };

        // Read beta and assert the returned data
        let call_data = readBetaCall::new((beta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readBetaCall::abi_decode_returns(&return_data).unwrap();
        let expected_value = if tto { 102 } else { 101 };
        let beta_expected = Beta::abi_encode(&Beta {
            id: UID {
                id: ID { bytes: beta_id },
            },
            a: Alpha {
                id: UID {
                    id: ID { bytes: alpha_id },
                },
                value: expected_value,
            },
        });
        assert_eq!(Beta::abi_encode(&return_data), beta_expected);
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        // Delete beta and assert the storage is empty afterwards
        let call_data = deleteBetaCall::new((beta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_creating_and_deleting_gamma(runtime: RuntimeSandbox, #[case] tto: bool) {
        let (alpha_id, beta_id, gamma_id) = if tto {
            // Create beta first for TTO method
            let call_data = createBetaCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let alpha_id = runtime.obtain_uid();
            let beta_id = runtime.obtain_uid();

            // Create gamma, passing beta as argument to be wrapped in it
            let call_data = createGammaTtoCall::new((beta_id,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let gamma_id = runtime.obtain_uid();

            (alpha_id, beta_id, gamma_id)
        } else {
            // Create gamma directly
            let call_data = createGammaCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            // Get the object ids
            let alpha_id = runtime.obtain_uid();
            let beta_id = runtime.obtain_uid();
            let gamma_id = runtime.obtain_uid();

            (alpha_id, beta_id, gamma_id)
        };

        // Read gamma and assert the returned data
        let call_data = readGammaCall::new((gamma_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readGammaCall::abi_decode_returns(&return_data).unwrap();
        let gamma_expected = Gamma::abi_encode(&Gamma {
            id: UID {
                id: ID { bytes: gamma_id },
            },
            a: Beta {
                id: UID {
                    id: ID { bytes: beta_id },
                },
                a: Alpha {
                    id: UID {
                        id: ID { bytes: alpha_id },
                    },
                    value: 101,
                },
            },
        });
        assert_eq!(Gamma::abi_encode(&return_data), gamma_expected);
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        // Delete gamma and assert the storage is empty afterwards
        let call_data = deleteGammaCall::new((gamma_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_creating_and_deleting_delta(runtime: RuntimeSandbox, #[case] tto: bool) {
        let (alpha_1_id, alpha_2_id, delta_id) = if tto {
            // Create alphas first for TTO method
            let call_data = createAlphaCall::new((101,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let alpha_1_id = runtime.obtain_uid();

            let call_data = createAlphaCall::new((102,)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let alpha_2_id = runtime.obtain_uid();

            // Create delta using TTO method
            let call_data = createDeltaTtoCall::new((alpha_1_id, alpha_2_id)).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let delta_id = runtime.obtain_uid();

            (alpha_1_id, alpha_2_id, delta_id)
        } else {
            // Create delta directly
            let call_data = createDeltaCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);

            let alpha_1_id = runtime.obtain_uid();
            let alpha_2_id = runtime.obtain_uid();
            let delta_id = runtime.obtain_uid();

            (alpha_1_id, alpha_2_id, delta_id)
        };

        // Read delta and assert the returned data
        let call_data = readDeltaCall::new((delta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
        let delta_expected = Delta::abi_encode(&Delta {
            id: UID {
                id: ID { bytes: delta_id },
            },
            a: vec![
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_1_id },
                    },
                    value: 101,
                },
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_2_id },
                    },
                    value: 102,
                },
            ],
        });
        assert_eq!(Delta::abi_encode(&return_data), delta_expected);
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        // Delete delta and assert the storage is empty afterwards
        let call_data = deleteDeltaCall::new((delta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    #[case(false)]
    #[case(true)]
    fn test_creating_and_deleting_epsilon(runtime: RuntimeSandbox, #[case] tto: bool) {
        let (alpha_1_id, alpha_2_id, alpha_3_id, alpha_4_id, delta_1_id, delta_2_id, epsilon_id) =
            if tto {
                let call_data = createAlphaCall::new((101,)).abi_encode();
                let (result, _) = runtime.call_entrypoint(call_data).unwrap();
                assert_eq!(0, result);

                let alpha_1_id = runtime.obtain_uid();

                let call_data = createAlphaCall::new((102,)).abi_encode();
                let (result, _) = runtime.call_entrypoint(call_data).unwrap();
                assert_eq!(0, result);

                let alpha_2_id = runtime.obtain_uid();

                // Create deltas first for TTO method
                let call_data = createDeltaTtoCall::new((alpha_1_id, alpha_2_id)).abi_encode();
                let (result, _) = runtime.call_entrypoint(call_data).unwrap();
                assert_eq!(0, result);

                let delta_1_id = runtime.obtain_uid();

                let call_data = createAlphaCall::new((103,)).abi_encode();
                let (result, _) = runtime.call_entrypoint(call_data).unwrap();
                assert_eq!(0, result);

                let alpha_3_id = runtime.obtain_uid();

                let call_data = createAlphaCall::new((104,)).abi_encode();
                let (result, _) = runtime.call_entrypoint(call_data).unwrap();
                assert_eq!(0, result);

                let alpha_4_id = runtime.obtain_uid();

                let call_data = createDeltaTtoCall::new((alpha_3_id, alpha_4_id)).abi_encode();
                let (result, _) = runtime.call_entrypoint(call_data).unwrap();
                assert_eq!(0, result);

                let delta_2_id = runtime.obtain_uid();

                // Create epsilon using TTO method
                let call_data = createEpsilonTtoCall::new((delta_1_id, delta_2_id)).abi_encode();
                let (result, _) = runtime.call_entrypoint(call_data).unwrap();
                assert_eq!(0, result);

                let epsilon_id = runtime.obtain_uid();

                (
                    alpha_1_id, alpha_2_id, alpha_3_id, alpha_4_id, delta_1_id, delta_2_id,
                    epsilon_id,
                )
            } else {
                // Create epsilon directly
                let call_data = createEpsilonCall::new(()).abi_encode();
                let (result, _) = runtime.call_entrypoint(call_data).unwrap();
                assert_eq!(0, result);

                let delta_1_id = runtime.obtain_uid();
                let alpha_1_id = runtime.obtain_uid();
                let alpha_2_id = runtime.obtain_uid();
                let delta_2_id = runtime.obtain_uid();
                let alpha_3_id = runtime.obtain_uid();
                let alpha_4_id = runtime.obtain_uid();
                let epsilon_id = runtime.obtain_uid();

                (
                    alpha_1_id, alpha_2_id, alpha_3_id, alpha_4_id, delta_1_id, delta_2_id,
                    epsilon_id,
                )
            };

        // Read epsilon and assert the returned data
        let call_data = readEpsilonCall::new((epsilon_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readEpsilonCall::abi_decode_returns(&return_data).unwrap();
        let epsilon_expected = Epsilon::abi_encode(&Epsilon {
            id: UID {
                id: ID { bytes: epsilon_id },
            },
            a: vec![
                Delta {
                    id: UID {
                        id: ID { bytes: delta_1_id },
                    },
                    a: vec![
                        Alpha {
                            id: UID {
                                id: ID { bytes: alpha_1_id },
                            },
                            value: 101,
                        },
                        Alpha {
                            id: UID {
                                id: ID { bytes: alpha_2_id },
                            },
                            value: 102,
                        },
                    ],
                },
                Delta {
                    id: UID {
                        id: ID { bytes: delta_2_id },
                    },
                    a: vec![
                        Alpha {
                            id: UID {
                                id: ID { bytes: alpha_3_id },
                            },
                            value: 103,
                        },
                        Alpha {
                            id: UID {
                                id: ID { bytes: alpha_4_id },
                            },
                            value: 104,
                        },
                    ],
                },
            ],
        });
        assert_eq!(Epsilon::abi_encode(&return_data), epsilon_expected);
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        // Delete epsilon and assert the storage is empty afterwards
        let call_data = deleteEpsilonCall::new((epsilon_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    const RECIPIENT_ADDRESS: [u8; 20] =
        [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1];

    #[rstest]
    fn test_transferring_beta(runtime: RuntimeSandbox) {
        let call_data = createBetaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_id = runtime.obtain_uid();
        let beta_id = runtime.obtain_uid();

        let beta_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &beta_id.0);

        // Transfer beta to the recipient
        let call_data = transferBetaCall::new((beta_id, RECIPIENT_ADDRESS.into())).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read beta from the recipient namespace in storage
        runtime.set_tx_origin(RECIPIENT_ADDRESS);
        let call_data = readBetaCall::new((beta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readBetaCall::abi_decode_returns(&return_data).unwrap();
        let beta_expected = Beta::abi_encode(&Beta {
            id: UID {
                id: ID { bytes: beta_id },
            },
            a: Alpha {
                id: UID {
                    id: ID { bytes: alpha_id },
                },
                value: 101,
            },
        });
        assert_eq!(Beta::abi_encode(&return_data), beta_expected);
        assert_eq!(0, result);

        // Assert that beta is not in the original namespace anymore
        assert_eq!(
            runtime.get_storage_at_slot(beta_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );
        assert_eq!(
            runtime.get_storage_at_slot(get_next_slot(&beta_slot.0)),
            [0u8; 32],
            "Slot should be empty"
        );
    }

    #[rstest]
    fn test_transferring_gamma(runtime: RuntimeSandbox) {
        let call_data = createGammaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_id = runtime.obtain_uid();
        let beta_id = runtime.obtain_uid();
        let gamma_id = runtime.obtain_uid();

        let gamma_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &gamma_id.0);

        // Transfer beta to the recipient
        let call_data = transferGammaCall::new((gamma_id, RECIPIENT_ADDRESS.into())).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read beta from the recipient namespace in storage
        runtime.set_tx_origin(RECIPIENT_ADDRESS);
        let call_data = readGammaCall::new((gamma_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readGammaCall::abi_decode_returns(&return_data).unwrap();
        let gamma_expected = Gamma::abi_encode(&Gamma {
            id: UID {
                id: ID { bytes: gamma_id },
            },
            a: Beta {
                id: UID {
                    id: ID { bytes: beta_id },
                },
                a: Alpha {
                    id: UID {
                        id: ID { bytes: alpha_id },
                    },
                    value: 101,
                },
            },
        });
        assert_eq!(Gamma::abi_encode(&return_data), gamma_expected);
        assert_eq!(0, result);

        // Assert that beta is not in the original namespace anymore
        assert_eq!(
            runtime.get_storage_at_slot(gamma_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );
        assert_eq!(
            runtime.get_storage_at_slot(get_next_slot(&gamma_slot.0)),
            [0u8; 32],
            "Slot should be empty"
        );
    }

    #[rstest]
    fn test_transferring_delta(runtime: RuntimeSandbox) {
        let call_data = createDeltaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_1_id = runtime.obtain_uid();
        let alpha_2_id = runtime.obtain_uid();
        let delta_id = runtime.obtain_uid();

        let delta_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &delta_id.0);

        let call_data = transferDeltaCall::new((delta_id, RECIPIENT_ADDRESS.into())).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        runtime.set_tx_origin(RECIPIENT_ADDRESS);
        // Read delta and assert the returned data
        let call_data = readDeltaCall::new((delta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
        let delta_expected = Delta::abi_encode(&Delta {
            id: UID {
                id: ID { bytes: delta_id },
            },
            a: vec![
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_1_id },
                    },
                    value: 101,
                },
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_2_id },
                    },
                    value: 102,
                },
            ],
        });
        assert_eq!(Delta::abi_encode(&return_data), delta_expected);
        assert_eq!(0, result);

        // Assert delta was deleted from the original namespace
        assert_eq!(
            runtime.get_storage_at_slot(delta_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );
        assert_eq!(
            runtime.get_storage_at_slot(get_next_slot(&delta_slot.0)),
            [0u8; 32],
            "Slot should be empty"
        );
    }
    #[rstest]
    fn test_rebuilding_gamma(runtime: RuntimeSandbox) {
        let call_data = createGammaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_id = runtime.obtain_uid();
        let beta_id = runtime.obtain_uid();
        let gamma_id = runtime.obtain_uid();

        let gamma_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &gamma_id.0);

        // Rebuild gamma
        let call_data = rebuildGammaCall::new((gamma_id, RECIPIENT_ADDRESS.into())).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let new_gamma_id = runtime.obtain_uid();

        // Read gamma from the recipient namespace in storage
        runtime.set_tx_origin(RECIPIENT_ADDRESS);
        let call_data = readGammaCall::new((new_gamma_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readGammaCall::abi_decode_returns(&return_data).unwrap();
        let gamma_expected = Gamma::abi_encode(&Gamma {
            id: UID {
                id: ID {
                    bytes: new_gamma_id,
                },
            },
            a: Beta {
                id: UID {
                    id: ID { bytes: beta_id },
                },
                a: Alpha {
                    id: UID {
                        id: ID { bytes: alpha_id },
                    },
                    value: 101,
                },
            },
        });
        assert_eq!(Gamma::abi_encode(&return_data), gamma_expected);
        assert_eq!(0, result);

        // Assert the old gamma was deleted from the original namespace
        assert_eq!(
            runtime.get_storage_at_slot(gamma_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );
        assert_eq!(
            runtime.get_storage_at_slot(get_next_slot(&gamma_slot.0)),
            [0u8; 32],
            "Slot should be empty"
        );
    }

    #[rstest]
    fn test_destruct_delta_to_beta(runtime: RuntimeSandbox) {
        let call_data = createDeltaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_1_id = runtime.obtain_uid();
        let alpha_2_id = runtime.obtain_uid();
        let delta_id = runtime.obtain_uid();

        let storage_before_destruct = runtime.get_storage();

        let call_data = destructDeltaToBetaCall::new((delta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_destruct = runtime.get_storage();

        // Delta is deleted and each alpha is wrapped in a new beta, hence all the original slots should be empty
        assert_empty_storage(&storage_before_destruct, &storage_after_destruct);

        let beta_1_id = runtime.obtain_uid();
        let beta_2_id = runtime.obtain_uid();

        // Read the betas and assert the returned data is correct
        let call_data = readBetaCall::new((beta_1_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readBetaCall::abi_decode_returns(&return_data).unwrap();
        let beta_expected = Beta::abi_encode(&Beta {
            id: UID {
                id: ID { bytes: beta_1_id },
            },
            a: Alpha {
                id: UID {
                    id: ID { bytes: alpha_2_id },
                },
                value: 102,
            },
        });
        assert_eq!(Beta::abi_encode(&return_data), beta_expected);
        assert_eq!(0, result);

        let call_data = readBetaCall::new((beta_2_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readBetaCall::abi_decode_returns(&return_data).unwrap();
        let beta_expected = Beta::abi_encode(&Beta {
            id: UID {
                id: ID { bytes: beta_2_id },
            },
            a: Alpha {
                id: UID {
                    id: ID { bytes: alpha_1_id },
                },
                value: 101,
            },
        });
        assert_eq!(Beta::abi_encode(&return_data), beta_expected);
        assert_eq!(0, result);
    }

    #[rstest]
    fn test_pushing_alpha_into_delta(runtime: RuntimeSandbox) {
        // Create empty delta
        let call_data = createEmptyDeltaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let delta_id = runtime.obtain_uid();

        // Create alpha
        let call_data = createAlphaCall::new((101,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_1_id = runtime.obtain_uid();
        let alpha_1_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_1_id.0);

        // Push alpha to delta
        let call_data = pushAlphaToDeltaCall::new((delta_id, alpha_1_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read delta and assert the returned data is correct
        let call_data = readDeltaCall::new((delta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
        let delta_expected = Delta::abi_encode(&Delta {
            id: UID {
                id: ID { bytes: delta_id },
            },
            a: vec![Alpha {
                id: UID {
                    id: ID { bytes: alpha_1_id },
                },
                value: 101,
            }],
        });
        assert_eq!(Delta::abi_encode(&return_data), delta_expected);
        assert_eq!(0, result);

        // Assert alpha is deleted from the original namespace
        assert_eq!(
            runtime.get_storage_at_slot(alpha_1_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );

        // Create second alpha
        let call_data = createAlphaCall::new((102,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_2_id = runtime.obtain_uid();
        let alpha_2_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_2_id.0);

        // Push second alpha to delta
        let call_data = pushAlphaToDeltaCall::new((delta_id, alpha_2_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read delta and assert the returned data is correct
        let call_data = readDeltaCall::new((delta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
        let delta_expected = Delta::abi_encode(&Delta {
            id: UID {
                id: ID { bytes: delta_id },
            },
            a: vec![
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_1_id },
                    },
                    value: 101,
                },
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_2_id },
                    },
                    value: 102,
                },
            ],
        });
        assert_eq!(Delta::abi_encode(&return_data), delta_expected);
        assert_eq!(0, result);

        // Assert second alpha is deleted from the original namespace
        assert_eq!(
            runtime.get_storage_at_slot(alpha_2_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );

        // Pop one alpha from delta and assert the returned data is correct
        let call_data = popAlphaFromDeltaCall::new((delta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read alpha_2 from the shared namespace and assert the data is correct
        let alpha_2_shared_slot = derive_object_slot(&SHARED, &alpha_2_id.0);
        assert_eq!(
            runtime.get_storage_at_slot(alpha_2_shared_slot.0),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 102, 198, 1,
                192, 204, 10, 101, 122, 43
            ],
            "Slot should not be empty"
        );
        /*
         *
            U256::from_str_radix("ebc75c981e6b1aa3", 16)
                .unwrap()
                .to_be_bytes(),
        */

        // Read delta after the pop and assert the data is correct
        let call_data = readDeltaCall::new((delta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
        let delta_expected = Delta::abi_encode(&Delta {
            id: UID {
                id: ID { bytes: delta_id },
            },
            a: vec![Alpha {
                id: UID {
                    id: ID { bytes: alpha_1_id },
                },
                value: 101,
            }],
        });
        assert_eq!(Delta::abi_encode(&return_data), delta_expected);
        assert_eq!(0, result);

        // Pop the last alpha from delta and assert the data is correct
        // In this case the beta vector is left empty.
        let call_data = popAlphaFromDeltaCall::new((delta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read alpha_1 from the shared namespace and assert the data is correct
        let alpha_1_shared_slot = derive_object_slot(&SHARED, &alpha_1_id.0);
        assert_eq!(
            runtime.get_storage_at_slot(alpha_1_shared_slot.0),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 101, 198, 1,
                192, 204, 10, 101, 122, 43
            ],
            "Slot should not be empty"
        );

        // Read the popped alpha and assert the returned data is correct
        let call_data = readAlphaCall::new((alpha_1_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readAlphaCall::abi_decode_returns(&return_data).unwrap();
        let alpha_expected = Alpha::abi_encode(&Alpha {
            id: UID {
                id: ID { bytes: alpha_1_id },
            },
            value: 101,
        });
        assert_eq!(Alpha::abi_encode(&return_data), alpha_expected);
        assert_eq!(0, result);

        // Read delta after the pop and assert the data is correct
        let call_data = readDeltaCall::new((delta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
        let delta_expected = Delta::abi_encode(&Delta {
            id: UID {
                id: ID { bytes: delta_id },
            },
            a: vec![],
        });
        assert_eq!(Delta::abi_encode(&return_data), delta_expected);
        assert_eq!(0, result);

        // Create third alpha
        let call_data = createAlphaCall::new((103,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_3_id = runtime.obtain_uid();

        // Push one more alpha to delta
        let call_data = pushAlphaToDeltaCall::new((delta_id, alpha_3_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_before_delete = runtime.get_storage();

        // Delete the shared alphas
        let call_data = deleteAlphaCall::new((alpha_1_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = deleteAlphaCall::new((alpha_2_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Delete delta
        let call_data = deleteDeltaCall::new((delta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();

        // Assert that all storage slots are empty except for the specified key
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_destruct_epsilon(runtime: RuntimeSandbox) {
        let call_data = createEpsilonCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let delta_1_id = runtime.obtain_uid();
        let alpha_1_id = runtime.obtain_uid();
        let alpha_2_id = runtime.obtain_uid();
        let delta_2_id = runtime.obtain_uid();
        let alpha_3_id = runtime.obtain_uid();
        let alpha_4_id = runtime.obtain_uid();
        let epsilon_id = runtime.obtain_uid();

        let epsilon_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &epsilon_id.0);

        let call_data = createAlphaCall::new((105,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_5_id = runtime.obtain_uid();
        let alpha_5_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_5_id.0);

        let call_data = destructEpsilonCall::new((epsilon_id, alpha_5_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Assert that epsilon is deleted from the original namespace
        assert_eq!(
            runtime.get_storage_at_slot(epsilon_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );

        // Assert that alpha 5 is deleted from the original namespace
        assert_eq!(
            runtime.get_storage_at_slot(alpha_5_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );

        // Read delta and assert the returned data
        let call_data = readDeltaCall::new((delta_2_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readDeltaCall::abi_decode_returns(&return_data).unwrap();
        let delta_expected = Delta::abi_encode(&Delta {
            id: UID {
                id: ID { bytes: delta_2_id },
            },
            a: vec![
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_3_id },
                    },
                    value: 103,
                },
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_4_id },
                    },
                    value: 104,
                },
                Alpha {
                    id: UID {
                        id: ID { bytes: alpha_5_id },
                    },
                    value: 105,
                },
            ],
        });
        assert_eq!(Delta::abi_encode(&return_data), delta_expected);
        assert_eq!(0, result);

        let new_epsilon_id = runtime.obtain_uid();

        // Read epsilon and assert the returned data
        let call_data = readEpsilonCall::new((new_epsilon_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readEpsilonCall::abi_decode_returns(&return_data).unwrap();
        let epsilon_expected = Epsilon::abi_encode(&Epsilon {
            id: UID {
                id: ID {
                    bytes: new_epsilon_id,
                },
            },
            a: vec![Delta {
                id: UID {
                    id: ID { bytes: delta_1_id },
                },
                a: vec![
                    Alpha {
                        id: UID {
                            id: ID { bytes: alpha_1_id },
                        },
                        value: 101,
                    },
                    Alpha {
                        id: UID {
                            id: ID { bytes: alpha_2_id },
                        },
                        value: 102,
                    },
                ],
            }],
        });
        assert_eq!(Epsilon::abi_encode(&return_data), epsilon_expected);
        assert_eq!(0, result);
    }

    #[rstest]
    fn test_pushing_and_popping_alpha_from_zeta(runtime: RuntimeSandbox) {
        let call_data = createEmptyZetaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let zeta_id = runtime.obtain_uid();

        let call_data = readZetaCall::new((zeta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
        let zeta_expected = Zeta::abi_encode(&Zeta {
            id: UID {
                id: ID { bytes: zeta_id },
            },
            b: Astra { a: vec![] },
        });
        assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
        assert_eq!(0, result);

        // Create alpha 1 and alpha 2
        let call_data = createAlphaCall::new((101,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_1_id = runtime.obtain_uid();
        let alpha_1_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_1_id.0);

        let call_data = createAlphaCall::new((102,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_2_id = runtime.obtain_uid();
        let alpha_2_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_2_id.0);

        // Pushback alpha 1 and alpha 2 to zeta
        let call_data = pushAlphaToZetaCall::new((zeta_id, alpha_1_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = pushAlphaToZetaCall::new((zeta_id, alpha_2_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read zeta and assert the returned data
        let call_data = readZetaCall::new((zeta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
        let zeta_expected = Zeta::abi_encode(&Zeta {
            id: UID {
                id: ID { bytes: zeta_id },
            },
            b: Astra {
                a: vec![
                    Alpha {
                        id: UID {
                            id: ID { bytes: alpha_1_id },
                        },
                        value: 101,
                    },
                    Alpha {
                        id: UID {
                            id: ID { bytes: alpha_2_id },
                        },
                        value: 102,
                    },
                ],
            },
        });
        assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
        assert_eq!(0, result);

        // Assert that alpha 1 and alpha 2 are deleted from the original namespace
        assert_eq!(
            runtime.get_storage_at_slot(alpha_1_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );
        assert_eq!(
            runtime.get_storage_at_slot(alpha_2_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );

        // Popback the last alpha from zeta
        let call_data = popAlphaFromZetaCall::new((zeta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read zeta and assert the returned data
        let call_data = readZetaCall::new((zeta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
        let zeta_expected = Zeta::abi_encode(&Zeta {
            id: UID {
                id: ID { bytes: zeta_id },
            },
            b: Astra {
                a: vec![Alpha {
                    id: UID {
                        id: ID { bytes: alpha_1_id },
                    },
                    value: 101,
                }],
            },
        });
        assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
        assert_eq!(0, result);

        // Assert that alpha 2 is under the shared namespace now
        let alpha_2_shared_slot = derive_object_slot(&SHARED, &alpha_2_id.0);
        assert_ne!(
            runtime.get_storage_at_slot(alpha_2_shared_slot.0),
            [0u8; 32],
            "Slot should not be empty"
        );

        let storage_before_delete = runtime.get_storage();

        // Delete zeta
        let call_data = deleteZetaCall::new((zeta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Delete alpha 2
        let call_data = deleteAlphaCall::new((alpha_2_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_delete = runtime.get_storage();
        assert_empty_storage(&storage_before_delete, &storage_after_delete);
    }

    #[rstest]
    fn test_popping_from_empty_zeta(runtime: RuntimeSandbox) {
        let call_data = createEmptyZetaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let zeta_id = runtime.obtain_uid();

        let call_data = readZetaCall::new((zeta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
        let zeta_expected = Zeta::abi_encode(&Zeta {
            id: UID {
                id: ID { bytes: zeta_id },
            },
            b: Astra { a: vec![] },
        });
        assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
        assert_eq!(0, result);

        // Create alpha 1 and alpha 2
        let call_data = createAlphaCall::new((101,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let alpha_1_id = runtime.obtain_uid();
        let alpha_1_slot = derive_object_slot(&MSG_SENDER_ADDRESS, &alpha_1_id.0);

        let call_data = createAlphaCall::new((102,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Pushback alpha 1 to zeta
        let call_data = pushAlphaToZetaCall::new((zeta_id, alpha_1_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read zeta and assert the returned data
        let call_data = readZetaCall::new((zeta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
        let zeta_expected = Zeta::abi_encode(&Zeta {
            id: UID {
                id: ID { bytes: zeta_id },
            },
            b: Astra {
                a: vec![Alpha {
                    id: UID {
                        id: ID { bytes: alpha_1_id },
                    },
                    value: 101,
                }],
            },
        });
        assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
        assert_eq!(0, result);

        // Assert that alpha 1 and alpha 2 are deleted from the original namespace
        assert_eq!(
            runtime.get_storage_at_slot(alpha_1_slot.0),
            [0u8; 32],
            "Slot should be empty"
        );

        // Popback alpha from zeta
        let call_data = popAlphaFromZetaCall::new((zeta_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read zeta and assert the returned data
        let call_data = readZetaCall::new((zeta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readZetaCall::abi_decode_returns(&return_data).unwrap();
        let zeta_expected = Zeta::abi_encode(&Zeta {
            id: UID {
                id: ID { bytes: zeta_id },
            },
            b: Astra { a: vec![] },
        });
        assert_eq!(Zeta::abi_encode(&return_data), zeta_expected);
        assert_eq!(0, result);

        // Assert that alpha 2 is under the shared namespace now
        let alpha_2_shared_slot = derive_object_slot(&SHARED, &alpha_1_id.0);
        assert_ne!(
            runtime.get_storage_at_slot(alpha_2_shared_slot.0),
            [0u8; 32],
            "Slot should not be empty"
        );

        // Popback again, even though the vector is empty
        let call_data = popAlphaFromZetaCall::new((zeta_id,)).abi_encode();
        let result = runtime.call_entrypoint(call_data);
        assert!(result.is_err());
    }

    #[rstest]
    fn test_pushing_and_popping_from_bora(runtime: RuntimeSandbox) {
        let call_data = createEtaCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let eta_id = runtime.obtain_uid();

        let call_data = readEtaCall::new((eta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readEtaCall::abi_decode_returns(&return_data).unwrap();
        let eta_expected = Eta::abi_encode(&Eta {
            id: UID {
                id: ID { bytes: eta_id },
            },
            b: Bora {
                a: vec![],
                b: vec![],
            },
        });
        assert_eq!(Eta::abi_encode(&return_data), eta_expected);
        assert_eq!(0, result);

        // Push to bora
        let call_data = pushToBoraCall::new((eta_id, 1)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read bora and assert the returned data
        let call_data = readEtaCall::new((eta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readEtaCall::abi_decode_returns(&return_data).unwrap();
        let eta_expected = Eta::abi_encode(&Eta {
            id: UID {
                id: ID { bytes: eta_id },
            },
            b: Bora {
                a: vec![1],
                b: vec![vec![1, 2, 3]],
            },
        });
        assert_eq!(Eta::abi_encode(&return_data), eta_expected);
        assert_eq!(0, result);

        // Push to bora
        let call_data = pushToBoraCall::new((eta_id, 10)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read bora and assert the returned data
        let call_data = readEtaCall::new((eta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readEtaCall::abi_decode_returns(&return_data).unwrap();
        let eta_expected = Eta::abi_encode(&Eta {
            id: UID {
                id: ID { bytes: eta_id },
            },
            b: Bora {
                a: vec![1, 10],
                b: vec![vec![1, 2, 3], vec![10, 11, 12]],
            },
        });
        assert_eq!(Eta::abi_encode(&return_data), eta_expected);
        assert_eq!(0, result);

        // Pop from bora
        let call_data = popFromBoraCall::new((eta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = popFromBoraCall::abi_decode_returns(&return_data).unwrap();
        let value = return_data._0;
        let vector = return_data._1;
        assert_eq!(value, 10);
        assert_eq!(vector, vec![10, 11, 12]);
        assert_eq!(0, result);

        let call_data = readEtaCall::new((eta_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = readEtaCall::abi_decode_returns(&return_data).unwrap();
        let eta_expected = Eta::abi_encode(&Eta {
            id: UID {
                id: ID { bytes: eta_id },
            },
            b: Bora {
                a: vec![1],
                b: vec![vec![1, 2, 3]],
            },
        });
        assert_eq!(Eta::abi_encode(&return_data), eta_expected);
        assert_eq!(0, result);
    }
}

mod dynamic_storage_fields {
    use alloy_primitives::address;
    use alloy_sol_types::{SolCall, SolValue, sol};

    use super::*;

    // NOTE: we can't use this fixture as #[once] because in order to catch events, we use an mpsc
    // channel. If we use this as #[once], there's a possibility this runtime is used in more than one
    // thread. If that happens, messages from test A can be received by test B.
    // Using once instance per thread assures this won't happen.
    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "dynamic_fields";
        const SOURCE_PATH: &str = "tests/storage/dynamic_fields.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function createFoo() public view;
        function createFooOwned() public view;
        function attachDynamicField(bytes32 foo, string name, uint64 value) public view;
        function readDynamicField(bytes32 foo, string name) public view returns (uint64);
        function dynamicFieldExists(bytes32 foo, string name) public view returns (bool);
        function mutateDynamicField(bytes32 foo, string name) public view;
        function mutateDynamicFieldTwo(bytes32 foo, string name, string name2) public view;
        function removeDynamicField(bytes32 foo, string name) public view returns (uint64);
        function attachDynamicFieldAddrU256(bytes32 foo, address name, uint256 value) public view;
        function readDynamicFieldAddrU256(bytes32 foo, address name) public view returns (uint256);
        function dynamicFieldExistsAddrU256(bytes32 foo, address name) public view returns (bool);
        function removeDynamicFieldAddrU256(bytes32 foo, address name) public view returns (uint64);
    );

    #[rstest]
    #[case(true)]
    #[case(false)]
    fn test_dynamic_fields(runtime: RuntimeSandbox, #[case] owned: bool) {
        if owned {
            runtime.set_msg_sender(SIGNER_ADDRESS);
            let call_data = createFooOwnedCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);
        } else {
            let call_data = createFooCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);
        }

        // Read the object id emmited from the contract's events
        let object_id = runtime.obtain_uid();

        let field_name_1 = "test_key_1".to_owned();
        let field_name_2 = "test_key_2".to_owned();

        let field_name_3 = address!("0x1234567890abcdef1234567890abcdef12345678");
        let field_name_4 = address!("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");

        // Check existence of dynamic fields before attaching them
        let call_data = dynamicFieldExistsCall::new((object_id, field_name_1.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsCall::new((object_id, field_name_2.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((object_id, field_name_3)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((object_id, field_name_4)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        // Attach a dynamic fields
        let call_data =
            attachDynamicFieldCall::new((object_id, field_name_1.clone(), 42)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data =
            attachDynamicFieldCall::new((object_id, field_name_2.clone(), 84)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data =
            attachDynamicFieldAddrU256Call::new((object_id, field_name_3, U256::from(u128::MAX)))
                .abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data =
            attachDynamicFieldAddrU256Call::new((object_id, field_name_4, U256::MAX)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the dynamic fields
        let call_data = readDynamicFieldCall::new((object_id, field_name_1.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(42u64.abi_encode(), result_data);

        let call_data = readDynamicFieldCall::new((object_id, field_name_2.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(84u64.abi_encode(), result_data);

        let call_data = readDynamicFieldAddrU256Call::new((object_id, field_name_3)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(U256::from(u128::MAX).abi_encode(), result_data);

        let call_data = readDynamicFieldAddrU256Call::new((object_id, field_name_4)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(U256::MAX.abi_encode(), result_data);

        // Check existence of dynamic fields
        let call_data = dynamicFieldExistsCall::new((object_id, field_name_1.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        let call_data = dynamicFieldExistsCall::new((object_id, field_name_2.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((object_id, field_name_3)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((object_id, field_name_4)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        // Mutatate the values
        let call_data = mutateDynamicFieldCall::new((object_id, field_name_1.clone())).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = mutateDynamicFieldCall::new((object_id, field_name_2.clone())).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read modified dynamic fields
        let call_data = readDynamicFieldCall::new((object_id, field_name_1.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(43u64.abi_encode(), result_data);

        let call_data = readDynamicFieldCall::new((object_id, field_name_2.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(85u64.abi_encode(), result_data);

        // Mutate both in the same function
        let call_data =
            mutateDynamicFieldTwoCall::new((object_id, field_name_1.clone(), field_name_2.clone()))
                .abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read modified dynamic fields
        let call_data = readDynamicFieldCall::new((object_id, field_name_1.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(44u64.abi_encode(), result_data);

        let call_data = readDynamicFieldCall::new((object_id, field_name_2.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(86u64.abi_encode(), result_data);

        // Remove fields
        let call_data = removeDynamicFieldCall::new((object_id, field_name_1.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(44u64.abi_encode(), result_data);

        let call_data = removeDynamicFieldCall::new((object_id, field_name_2.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(86u64.abi_encode(), result_data);

        let call_data = removeDynamicFieldAddrU256Call::new((object_id, field_name_3)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(U256::from(u128::MAX).abi_encode(), result_data);

        let call_data = removeDynamicFieldAddrU256Call::new((object_id, field_name_4)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(U256::MAX.abi_encode(), result_data);

        // Check existence of dynamic fields
        let call_data = dynamicFieldExistsCall::new((object_id, field_name_1.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsCall::new((object_id, field_name_2.clone())).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((object_id, field_name_3)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((object_id, field_name_4)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);
    }
}

mod dynamic_storage_fields_named_id {
    use alloy_primitives::address;
    use alloy_sol_types::{SolCall, SolValue, sol};

    use super::*;

    // NOTE: we can't use this fixture as #[once] because in order to catch events, we use an mpsc
    // channel. If we use this as #[once], there's a possibility this runtime is used in more than one
    // thread. If that happens, messages from test A can be received by test B.
    // Using once instance per thread assures this won't happen.
    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "dynamic_fields_named_id";
        const SOURCE_PATH: &str = "tests/storage/dynamic_fields_named_id.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function createFoo() public view;
        function createFooOwned() public view;
        function attachDynamicField(string name, uint64 value) public view;
        function readDynamicField(string name) public view returns (uint64);
        function dynamicFieldExists(string name) public view returns (bool);
        function mutateDynamicField(string name) public view;
        function mutateDynamicFieldTwo(string name, string name2) public view;
        function removeDynamicField(string name) public view returns (uint64);
        function attachDynamicFieldAddrU256(address name, uint256 value) public view;
        function readDynamicFieldAddrU256(address name) public view returns (uint256);
        function dynamicFieldExistsAddrU256(address name) public view returns (bool);
        function removeDynamicFieldAddrU256(address name) public view returns (uint64);
    );

    #[rstest]
    #[case(true)]
    #[case(false)]
    fn test_dynamic_fields_named_id(runtime: RuntimeSandbox, #[case] owned: bool) {
        if owned {
            runtime.set_msg_sender(SIGNER_ADDRESS);
            let call_data = createFooOwnedCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);
        } else {
            let call_data = createFooCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);
        }

        let field_name_1 = "test_key_1".to_owned();
        let field_name_2 = "test_key_2".to_owned();

        let field_name_3 = address!("0x1234567890abcdef1234567890abcdef12345678");
        let field_name_4 = address!("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");

        // Check existence of dynamic fields before attaching them
        let call_data = dynamicFieldExistsCall::new((field_name_1.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsCall::new((field_name_2.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((field_name_3,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((field_name_4,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        // Attach a dynamic fields
        let call_data = attachDynamicFieldCall::new((field_name_1.clone(), 42)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = attachDynamicFieldCall::new((field_name_2.clone(), 84)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data =
            attachDynamicFieldAddrU256Call::new((field_name_3, U256::from(u128::MAX))).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = attachDynamicFieldAddrU256Call::new((field_name_4, U256::MAX)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read the dynamic fields
        let call_data = readDynamicFieldCall::new((field_name_1.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(42u64.abi_encode(), result_data);

        let call_data = readDynamicFieldCall::new((field_name_2.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(84u64.abi_encode(), result_data);

        let call_data = readDynamicFieldAddrU256Call::new((field_name_3,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(U256::from(u128::MAX).abi_encode(), result_data);

        let call_data = readDynamicFieldAddrU256Call::new((field_name_4,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(U256::MAX.abi_encode(), result_data);

        // Check existence of dynamic fields
        let call_data = dynamicFieldExistsCall::new((field_name_1.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        let call_data = dynamicFieldExistsCall::new((field_name_2.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((field_name_3,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((field_name_4,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        // Mutatate the values
        let call_data = mutateDynamicFieldCall::new((field_name_1.clone(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = mutateDynamicFieldCall::new((field_name_2.clone(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read modified dynamic fields
        let call_data = readDynamicFieldCall::new((field_name_1.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(43u64.abi_encode(), result_data);

        let call_data = readDynamicFieldCall::new((field_name_2.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(85u64.abi_encode(), result_data);

        // Mutate both in the same function
        let call_data =
            mutateDynamicFieldTwoCall::new((field_name_1.clone(), field_name_2.clone()))
                .abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read modified dynamic fields
        let call_data = readDynamicFieldCall::new((field_name_1.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(44u64.abi_encode(), result_data);

        let call_data = readDynamicFieldCall::new((field_name_2.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(86u64.abi_encode(), result_data);

        // Remove fields
        let call_data = removeDynamicFieldCall::new((field_name_1.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(44u64.abi_encode(), result_data);

        let call_data = removeDynamicFieldCall::new((field_name_2.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(86u64.abi_encode(), result_data);

        let call_data = removeDynamicFieldAddrU256Call::new((field_name_3,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(U256::from(u128::MAX).abi_encode(), result_data);

        let call_data = removeDynamicFieldAddrU256Call::new((field_name_4,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(U256::MAX.abi_encode(), result_data);

        // Check existence of dynamic fields
        let call_data = dynamicFieldExistsCall::new((field_name_1.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsCall::new((field_name_2.clone(),)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((field_name_3,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = dynamicFieldExistsAddrU256Call::new((field_name_4,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);
    }
}

mod dynamic_table {
    use alloy_primitives::address;
    use alloy_sol_types::{SolCall, SolValue, sol};

    use super::*;

    // NOTE: we can't use this fixture as #[once] because in order to catch events, we use an mpsc
    // channel. If we use this as #[once], there's a possibility this runtime is used in more than one
    // thread. If that happens, messages from test A can be received by test B.
    // Using once instance per thread assures this won't happen.
    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "dynamic_table";
        const SOURCE_PATH: &str = "tests/storage/dynamic_table.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]

        struct String {
            uint8[] bytes;
        }

        function createFoo() public view;
        function createFooOwned() public view;
        function attachTable(bytes32 foo) public view;
        function createEntry(bytes32 foo, address key, uint64 value) public view;
        function containsEntry(bytes32 foo, address key) public view returns (bool);
        function removeEntry(bytes32 foo, address key) public view returns (uint64);
        function readTableEntryValue(bytes32 foo, address key) public view returns (uint64);
        function mutateTableEntry(bytes32 foo, address key) public view;
        function mutateTwoEntryValues(bytes32 foo, address key, address key2) public view;
    );

    #[rstest]
    #[case(true)]
    #[case(false)]
    fn test_dynamic_table(runtime: RuntimeSandbox, #[case] owned: bool) {
        if owned {
            runtime.set_msg_sender(SIGNER_ADDRESS);
            let call_data = createFooOwnedCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);
        } else {
            let call_data = createFooCall::new(()).abi_encode();
            let (result, _) = runtime.call_entrypoint(call_data).unwrap();
            assert_eq!(0, result);
        }

        // Read the object id emmited from the contract's events
        let object_id = runtime.obtain_uid();

        let key_1 = address!("0x1234567890abcdef1234567890abcdef12345678");
        let key_2 = address!("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd");

        // Attach the table
        let call_data = attachTableCall::new((object_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Check entries we are going to create do not exist
        let call_data = containsEntryCall::new((object_id, key_1)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = containsEntryCall::new((object_id, key_2)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        // Create entry
        let call_data = createEntryCall::new((object_id, key_1, 42)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = createEntryCall::new((object_id, key_2, 84)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Check entries we are going to create exist
        let call_data = containsEntryCall::new((object_id, key_1)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        let call_data = containsEntryCall::new((object_id, key_2)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        // Read recently created entries
        let call_data = readTableEntryValueCall::new((object_id, key_1)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(42.abi_encode(), result_data);

        let call_data = readTableEntryValueCall::new((object_id, key_2)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(84.abi_encode(), result_data);

        // Mutate entries individually
        let call_data = mutateTableEntryCall::new((object_id, key_1)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = mutateTableEntryCall::new((object_id, key_2)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read recently mutated entries
        let call_data = readTableEntryValueCall::new((object_id, key_1)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(43.abi_encode(), result_data);

        let call_data = readTableEntryValueCall::new((object_id, key_2)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(85.abi_encode(), result_data);

        // Mutate both entries simultaneusly
        let call_data = mutateTwoEntryValuesCall::new((object_id, key_1, key_2)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Read recently mutated entries
        let call_data = readTableEntryValueCall::new((object_id, key_1)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(44.abi_encode(), result_data);

        let call_data = readTableEntryValueCall::new((object_id, key_2)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(86.abi_encode(), result_data);

        // Remove entries
        let call_data = removeEntryCall::new((object_id, key_1)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(44.abi_encode(), result_data);

        let call_data = removeEntryCall::new((object_id, key_2)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(86.abi_encode(), result_data);

        // Check entries we just deleted do not exist
        let call_data = containsEntryCall::new((object_id, key_1)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);

        let call_data = containsEntryCall::new((object_id, key_2)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(false.abi_encode(), result_data);
    }
}

mod erc20 {
    use alloy_primitives::address;
    use alloy_sol_types::{SolCall, SolValue, sol};

    use super::*;

    // NOTE: we can't use this fixture as #[once] because in order to catch events, we use an mpsc
    // channel. If we use this as #[once], there's a possibility this runtime is used in more than one
    // thread. If that happens, messages from test A can be received by test B.
    // Using once instance per thread assures this won't happen.
    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "erc20";
        const SOURCE_PATH: &str = "tests/storage/erc20.move";

        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]

        struct String {
            uint8[] bytes;
        }


        function mint(address to, uint256 amount) external view;
        function create() public view;
        function burn(address from, uint256 amount) external view;
        function balanceOf(address address) public view returns (uint256);
        function totalSupply() external view returns (uint256);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
    );

    #[rstest]
    fn test_erc20(runtime: RuntimeSandbox) {
        let address_1 = address!("0xcafecafecafecafecafecafecafecafecafecafe");
        runtime.set_msg_sender(**address_1);
        runtime.set_tx_origin(**address_1);
        let address_2 = address!("0xbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef");
        let address_3 = address!("0xabcabcabcabcabcabcabcabcabcabcabcabcabca");

        // Create the contract
        let call_data = createCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Check frozen info
        let call_data = decimalsCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(18.abi_encode(), result_data);

        let call_data = nameCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!("Test Coin".abi_encode(), result_data);

        let call_data = symbolCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!("TST".abi_encode(), result_data);

        // Mint new coins
        let call_data = totalSupplyCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(0.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_1,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(0.abi_encode(), result_data);

        let call_data = mintCall::new((address_1, U256::from(9999999))).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = totalSupplyCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(9999999.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_1,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(9999999.abi_encode(), result_data);

        // Transfer
        let call_data = transferCall::new((address_2, U256::from(1111))).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(true.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_1,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(9998888.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_2,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(1111.abi_encode(), result_data);

        // Burn
        let call_data = totalSupplyCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(9999999.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_1,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(9998888.abi_encode(), result_data);

        let call_data = burnCall::new((address_1, U256::from(2222))).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = totalSupplyCall::new(()).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(9997777.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_1,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(9996666.abi_encode(), result_data);

        // Allowance
        // Allow address_1 to spend 100 TST from address_2
        let call_data = allowanceCall::new((address_2, address_1)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(0.abi_encode(), result_data);

        runtime.set_msg_sender(**address_2);
        runtime.set_tx_origin(**address_2);
        let call_data = approveCall::new((address_1, U256::from(100))).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        runtime.set_msg_sender(**address_1);
        runtime.set_tx_origin(**address_1);
        let call_data = allowanceCall::new((address_2, address_1)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(100.abi_encode(), result_data);

        // Transfer from
        // Transfer from address_2 100 TST using address_1 to address_3
        let call_data = balanceOfCall::new((address_1,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(9996666.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_2,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(1111.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_3,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(0.abi_encode(), result_data);

        let call_data = transferFromCall::new((address_2, address_3, U256::from(100))).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let call_data = balanceOfCall::new((address_1,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(9996666.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_2,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(1011.abi_encode(), result_data);

        let call_data = balanceOfCall::new((address_3,)).abi_encode();
        let (result, result_data) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        assert_eq!(100.abi_encode(), result_data);
    }
}

mod simple_warrior {
    use super::*;
    use crate::common::runtime_sandbox::constants::SIGNER_ADDRESS;
    use alloy_sol_types::{SolCall, SolValue, sol};

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "simple_warrior";
        const SOURCE_PATH: &str = "tests/storage/simple_warrior.move";

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

        struct OptionSword {
            Sword[] vec;
        }

        struct OptionShield {
            Shield[] vec;
        }

        struct Sword {
            UID id;
            uint8 strength;
        }

        struct Shield {
            UID id;
            uint8 armor;
        }

        struct Warrior {
            UID id;
            OptionSword sword;
            OptionShield shield;
            Faction faction;
        }

        enum Faction {
            Alliance,
            Horde,
            Rebel
        }

        function createWarrior() public view;
        function createSword(uint8 strength) public view;
        function createShield(uint8 armor) public view;
        function equipSword(bytes32 id, bytes32 sword) public;
        function equipShield(bytes32 id, bytes32 shield) public;
        function inspectWarrior(bytes32 id) public view returns (Warrior);
        function inspectSword(bytes32 id) public view returns (Sword);
        function inspectShield(bytes32 id) public view returns (Shield);
        function destroyWarrior(bytes32 id) public;
        function destroySword(bytes32 id) public;
        function changeFaction(bytes32 id, uint8 faction) public;
    );

    #[rstest]
    fn test_equip_warrior(runtime: RuntimeSandbox) {
        runtime.set_msg_sender(SIGNER_ADDRESS);

        // Create warrior
        let call_data = createWarriorCall::new(()).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let warrior_id = runtime.obtain_uid();

        // Inspect warrior and assert it has no sword or shield
        let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
        let expected_return_data = Warrior::abi_encode(&Warrior {
            id: UID {
                id: ID { bytes: warrior_id },
            },
            sword: OptionSword { vec: vec![] },
            shield: OptionShield { vec: vec![] },
            faction: Faction::Rebel,
        });
        assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
        assert_eq!(0, result);

        // Create sword
        let call_data = createSwordCall::new((66,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let sword_id = runtime.obtain_uid();
        let sword_slot = derive_object_slot(&SIGNER_ADDRESS, &sword_id.0);

        // Equip sword
        let call_data = equipSwordCall::new((warrior_id, sword_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Inspect warrior and assert it has the sword equiped
        let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
        let expected_return_data = Warrior::abi_encode(&Warrior {
            id: UID {
                id: ID { bytes: warrior_id },
            },
            sword: OptionSword {
                vec: vec![Sword {
                    id: UID {
                        id: ID { bytes: sword_id },
                    },
                    strength: 66,
                }],
            },
            shield: OptionShield { vec: vec![] },
            faction: Faction::Rebel,
        });
        assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
        assert_eq!(0, result);

        // Assert that the original sword slot (under the sender's address) is now empty
        assert_eq!(runtime.get_storage_at_slot(sword_slot.0), [0u8; 32]);
        // Create new sword
        let call_data = createSwordCall::new((77,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let new_sword_id = runtime.obtain_uid();
        let new_sword_slot = derive_object_slot(&SIGNER_ADDRESS, &new_sword_id.0);

        // Equip new sword
        let call_data = equipSwordCall::new((warrior_id, new_sword_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Inspect warrior and assert it has the new sword equiped
        let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
        let expected_return_data = Warrior::abi_encode(&Warrior {
            id: UID {
                id: ID { bytes: warrior_id },
            },
            sword: OptionSword {
                vec: vec![Sword {
                    id: UID {
                        id: ID {
                            bytes: new_sword_id,
                        },
                    },
                    strength: 77,
                }],
            },
            shield: OptionShield { vec: vec![] },
            faction: Faction::Rebel,
        });
        assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
        assert_eq!(0, result);

        // Assert that the original new sword slot (under the sender's address) is now empty
        assert_eq!(runtime.get_storage_at_slot(new_sword_slot.0), [0u8; 32]);

        // Assert that the original old sword slot (under the sender's address) holds the old sword now
        assert_ne!(runtime.get_storage_at_slot(sword_slot.0), [0u8; 32]);

        let call_data = inspectSwordCall::new((sword_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = inspectSwordCall::abi_decode_returns(&return_data).unwrap();
        let expected_return_data = Sword::abi_encode(&Sword {
            id: UID {
                id: ID { bytes: sword_id },
            },
            strength: 66,
        });
        assert_eq!(Sword::abi_encode(&return_data), expected_return_data);
        assert_eq!(0, result);

        // Create shield
        let call_data = createShieldCall::new((42,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let shield_id = runtime.obtain_uid();
        let shield_slot = derive_object_slot(&SIGNER_ADDRESS, &shield_id.0);

        let call_data = equipShieldCall::new((warrior_id, shield_id)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Inspect warrior and assert it has the shield equiped
        let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
        let expected_return_data = Warrior::abi_encode(&Warrior {
            id: UID {
                id: ID { bytes: warrior_id },
            },
            sword: OptionSword {
                vec: vec![Sword {
                    id: UID {
                        id: ID {
                            bytes: new_sword_id,
                        },
                    },
                    strength: 77,
                }],
            },
            shield: OptionShield {
                vec: vec![Shield {
                    id: UID {
                        id: ID { bytes: shield_id },
                    },
                    armor: 42,
                }],
            },
            faction: Faction::Rebel,
        });
        assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
        assert_eq!(0, result);

        // Assert that the original shield slot (under the sender's address) is now empty
        assert_eq!(runtime.get_storage_at_slot(shield_slot.0), [0u8; 32]);

        // Change faction
        let call_data = changeFactionCall::new((warrior_id, 0)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Inspect warrior and assert it has the new faction
        let call_data = inspectWarriorCall::new((warrior_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = inspectWarriorCall::abi_decode_returns(&return_data).unwrap();
        let expected_return_data = Warrior::abi_encode(&Warrior {
            id: UID {
                id: ID { bytes: warrior_id },
            },
            sword: OptionSword {
                vec: vec![Sword {
                    id: UID {
                        id: ID {
                            bytes: new_sword_id,
                        },
                    },
                    strength: 77,
                }],
            },
            shield: OptionShield {
                vec: vec![Shield {
                    id: UID {
                        id: ID { bytes: shield_id },
                    },
                    armor: 42,
                }],
            },
            faction: Faction::Alliance,
        });
        assert_eq!(Warrior::abi_encode(&return_data), expected_return_data);
        assert_eq!(0, result);

        let storage_before_destroy = runtime.get_storage();
        // Destroy warrior
        let call_data = destroyWarriorCall::new((warrior_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        // Destroy the old sword too, just to make the test simpler
        let call_data = destroySwordCall::new((sword_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let storage_after_destroy = runtime.get_storage();

        // Assert that the storage is empty
        assert_empty_storage(&storage_before_destroy, &storage_after_destroy);
    }
}

mod enums {
    use super::*;
    use crate::common::runtime_sandbox::constants::SIGNER_ADDRESS;
    use alloy_sol_types::{SolCall, SolValue, sol};

    #[fixture]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "enums";
        const SOURCE_PATH: &str = "tests/storage/enums.move";
        let mut translated_package =
            translate_test_package_with_framework(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[derive(Debug)]
        struct ID {
            bytes32 bytes;
        }

        #[derive(Debug)]
        struct UID {
            ID id;
        }

        #[allow(missing_docs)]
        #[derive(Debug, PartialEq)]
        enum Numbers {
            One,
            Two,
            Three,
        }

        #[allow(missing_docs)]
        #[derive(Debug, PartialEq)]
        enum Colors {
            Red,
            Green,
            Blue,
        }

        #[allow(missing_docs)]
        struct StructWithSimpleEnums {
            UID id;
            Numbers n;
            Colors c;
        }

        function createStructWithSimpleEnums(address recipient) public view;
        function getStructWithSimpleEnums(bytes32 id) public view returns (StructWithSimpleEnums);
        function setNumber(bytes32 id, Numbers n) public;
        function setColor(bytes32 id, Colors c) public;
        function getNumber(bytes32 id) public view returns (Numbers);
        function getColor(bytes32 id) public view returns (Colors);
        function destroyStructWithSimpleEnums(bytes32 id) public;

        function createFooStruct(address recipient) public view;
        function testFooStruct(bytes32 id) public view;
        function setVariantA(bytes32 id, uint16 x, uint32 y) public;
        function setVariantB(bytes32 id, uint64 x, uint128 y, bool z) public;
        function setVariantC(bytes32 id, Numbers n, Colors c) public;
        function getVariantA(bytes32 id) public view returns (uint16, uint32);
        function getVariantB(bytes32 id) public view returns (uint64, uint128, bool);
        function getVariantC(bytes32 id) public view returns (Numbers, Colors);
        function destroyFooStruct(bytes32 id) public;
    );

    #[rstest]
    fn test_create_struct_with_simple_enums(runtime: RuntimeSandbox) {
        runtime.set_msg_sender(SIGNER_ADDRESS);

        let call_data = createStructWithSimpleEnumsCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let struct_with_simple_enums_id = runtime.obtain_uid();

        let call_data =
            getStructWithSimpleEnumsCall::new((struct_with_simple_enums_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = getStructWithSimpleEnumsCall::abi_decode_returns(&return_data).unwrap();
        let expected_return_data = StructWithSimpleEnums::abi_encode(&StructWithSimpleEnums {
            id: UID {
                id: ID {
                    bytes: struct_with_simple_enums_id,
                },
            },
            n: Numbers::One,
            c: Colors::Red,
        });
        assert_eq!(
            StructWithSimpleEnums::abi_encode(&return_data),
            expected_return_data
        );
        assert_eq!(0, result);
        runtime.print_storage();

        let call_data =
            setNumberCall::new((struct_with_simple_enums_id, Numbers::Two)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        runtime.print_storage();

        let call_data =
            setColorCall::new((struct_with_simple_enums_id, Colors::Green)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        runtime.print_storage();

        let call_data = getNumberCall::new((struct_with_simple_enums_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = getNumberCall::abi_decode_returns(&return_data).unwrap();
        let expected_return_data = Numbers::abi_encode(&Numbers::Two);
        assert_eq!(Numbers::abi_encode(&return_data), expected_return_data);
        assert_eq!(0, result);

        let call_data = getColorCall::new((struct_with_simple_enums_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = getColorCall::abi_decode_returns(&return_data).unwrap();
        let expected_return_data = Colors::abi_encode(&Colors::Green);
        assert_eq!(Colors::abi_encode(&return_data), expected_return_data);
        assert_eq!(0, result);

        let storage_before_destroy = runtime.get_storage();
        let call_data =
            destroyStructWithSimpleEnumsCall::new((struct_with_simple_enums_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        let storage_after_destroy = runtime.get_storage();

        // Assert that the storage is empty
        assert_empty_storage(&storage_before_destroy, &storage_after_destroy);
    }

    #[rstest]
    fn test_create_foo_struct(runtime: RuntimeSandbox) {
        runtime.set_msg_sender(SIGNER_ADDRESS);

        let call_data = createFooStructCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        let foo_struct_id = runtime.obtain_uid();

        runtime.print_storage();

        let call_data = getVariantACall::new((foo_struct_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = getVariantACall::abi_decode_returns(&return_data).unwrap();
        let got = (return_data._0, return_data._1);
        assert_eq!(got, (1u16, 2u32));
        assert_eq!(0, result);

        let call_data = setVariantACall::new((foo_struct_id, 2, 3)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        runtime.print_storage();

        let call_data = getVariantACall::new((foo_struct_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = getVariantACall::abi_decode_returns(&return_data).unwrap();
        let got = (return_data._0, return_data._1);
        assert_eq!(got, (2u16, 3u32));
        assert_eq!(0, result);

        let call_data = setVariantBCall::new((foo_struct_id, 4, 5, true)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        runtime.print_storage();

        let call_data = getVariantBCall::new((foo_struct_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = getVariantBCall::abi_decode_returns(&return_data).unwrap();
        let got = (return_data._0, return_data._1, return_data._2);
        assert_eq!(got, (4u64, 5u128, true));
        assert_eq!(0, result);

        let call_data =
            setVariantCCall::new((foo_struct_id, Numbers::Two, Colors::Blue)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);

        runtime.print_storage();

        let call_data = getVariantCCall::new((foo_struct_id,)).abi_encode();
        let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
        let return_data = getVariantCCall::abi_decode_returns(&return_data).unwrap();
        let got = (return_data._0, return_data._1);
        assert_eq!(got, (Numbers::Two, Colors::Blue));
        assert_eq!(0, result);

        let storage_before_destroy = runtime.get_storage();
        let call_data = destroyFooStructCall::new((foo_struct_id,)).abi_encode();
        let (result, _) = runtime.call_entrypoint(call_data).unwrap();
        assert_eq!(0, result);
        let storage_after_destroy = runtime.get_storage();

        // Assert that the storage is empty
        assert_empty_storage(&storage_before_destroy, &storage_after_destroy);
    }
}
