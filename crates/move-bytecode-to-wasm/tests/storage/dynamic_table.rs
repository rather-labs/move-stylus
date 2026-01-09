use crate::common::runtime;
use alloy_primitives::address;
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::constants::SIGNER_ADDRESS;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

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
fn test_dynamic_table(
    #[with("dynamic_table", "tests/storage/move_sources/dynamic_table.move")]
    runtime: RuntimeSandbox,
    #[case] owned: bool,
) {
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
    let object_id = runtime.obtain_uid().unwrap();

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
