use crate::common::runtime;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::constants::SIGNER_ADDRESS;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

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
fn test_dynamic_fields(
    #[with("dynamic_fields", "tests/storage/move_sources/dynamic_fields.move")]
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
    let call_data = attachDynamicFieldCall::new((object_id, field_name_1.clone(), 42)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = attachDynamicFieldCall::new((object_id, field_name_2.clone(), 84)).abi_encode();
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
