use super::*;
use crate::common::runtime;
use alloy_primitives::address;
use alloy_primitives::{FixedBytes, U256, address, hex, keccak256};
use alloy_sol_types::sol;
use alloy_sol_types::{SolCall, SolValue, sol};
use move_test_runner::constants::MSG_SENDER_ADDRESS;
use move_test_runner::constants::SIGNER_ADDRESS;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::rstest;

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

    // StructWithSimpleEnums
    function createStructWithSimpleEnums(address recipient) public view;
    function getStructWithSimpleEnums(bytes32 id) public view returns (StructWithSimpleEnums);
    function setNumber(bytes32 id, Numbers n) public;
    function setColor(bytes32 id, Colors c) public;
    function getNumber(bytes32 id) public view returns (Numbers);
    function getColor(bytes32 id) public view returns (Colors);
    function destroyStructWithSimpleEnums(bytes32 id) public;

    // FooStruct
    function createFooStruct(address recipient) public view;
    function setVariantA(bytes32 id, uint16 x, uint32 y) public;
    function setVariantB(bytes32 id, uint64 x, uint128 y, bool z) public;
    function setVariantC(bytes32 id, Numbers n, Colors c) public;
    function getVariantA(bytes32 id) public view returns (uint16, uint32);
    function getVariantB(bytes32 id) public view returns (uint64, uint128, bool);
    function getVariantC(bytes32 id) public view returns (Numbers, Colors);
    function destroyFooStruct(bytes32 id) public;

    // BarStruct
    function createBarStruct(address recipient) public view;
    function getFooEnumVariantA(bytes32 id) public view returns (uint16, uint32);
    function getFooEnumVariantB(bytes32 id) public view returns (uint64, uint128, bool);
    function getFooEnumVariantC(bytes32 id) public view returns (Numbers, Colors);
    function setFooEnumVariantA(bytes32 id, uint16 x, uint32 y) public;
    function setFooEnumVariantB(bytes32 id, uint64 x, uint128 y, bool z) public;
    function setFooEnumVariantC(bytes32 id, Numbers n, Colors c) public;
    function getAddress(bytes32 id) public view returns (address);
    function destroyBarStruct(bytes32 id) public;

    // GenericBarStruct
    function createGenericBarStruct(address recipient) public view;
    function getGenericFooEnumVariantA(bytes32 id) public view returns (uint16, uint32);
    function getGenericFooEnumVariantB(bytes32 id) public view returns (uint64, uint128, bool);
    function getGenericFooEnumVariantC(bytes32 id) public view returns (Numbers, Colors);
    function setGenericFooEnumVariantA(bytes32 id, uint16 x, uint32 y) public;
    function setGenericFooEnumVariantB(bytes32 id, uint64 x, uint128 y, bool z) public;
    function setGenericFooEnumVariantC(bytes32 id, Numbers n, Colors c) public;
    function getGenericAddress(bytes32 id) public view returns (address);
    function destroyGenericBarStruct(bytes32 id) public;
);

#[rstest]
fn test_struct_with_simple_enums(
    #[with("enums", "tests/storage/move_sources/enums.move")] runtime: RuntimeSandbox,
) {
    runtime.set_msg_sender(SIGNER_ADDRESS);

    let call_data = createStructWithSimpleEnumsCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let struct_with_simple_enums_id = runtime.obtain_uid();

    let call_data = getStructWithSimpleEnumsCall::new((struct_with_simple_enums_id,)).abi_encode();
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

    let call_data = setNumberCall::new((struct_with_simple_enums_id, Numbers::Two)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = setColorCall::new((struct_with_simple_enums_id, Colors::Green)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

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
fn test_foo_struct(
    #[with("enums", "tests/storage/move_sources/enums.move")] runtime: RuntimeSandbox,
) {
    runtime.set_msg_sender(SIGNER_ADDRESS);

    let call_data = createFooStructCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let foo_struct_id = runtime.obtain_uid();

    let call_data = getVariantACall::new((foo_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getVariantACall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1);
    assert_eq!(got, (1u16, 2u32));
    assert_eq!(0, result);

    let call_data = setVariantACall::new((foo_struct_id, 2, 3)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = getVariantACall::new((foo_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getVariantACall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1);
    assert_eq!(got, (2u16, 3u32));
    assert_eq!(0, result);

    let call_data = setVariantBCall::new((foo_struct_id, 4, 5, true)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = getVariantBCall::new((foo_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getVariantBCall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1, return_data._2);
    assert_eq!(got, (4u64, 5u128, true));
    assert_eq!(0, result);

    let call_data = setVariantCCall::new((foo_struct_id, Numbers::Two, Colors::Blue)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

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

#[rstest]
fn test_bar_struct(
    #[with("enums", "tests/storage/move_sources/enums.move")] runtime: RuntimeSandbox,
) {
    runtime.set_msg_sender(SIGNER_ADDRESS);

    let call_data = createBarStructCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let bar_struct_id = runtime.obtain_uid();

    let call_data = getFooEnumVariantBCall::new((bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooEnumVariantBCall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1, return_data._2);
    assert_eq!(got, (42u64, 43u128, true));
    assert_eq!(0, result);

    let call_data = setFooEnumVariantACall::new((bar_struct_id, 2, 3)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = getFooEnumVariantACall::new((bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooEnumVariantACall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1);
    assert_eq!(got, (2u16, 3u32));
    assert_eq!(0, result);

    let call_data = setFooEnumVariantBCall::new((bar_struct_id, 4, 5, true)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = getFooEnumVariantBCall::new((bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooEnumVariantBCall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1, return_data._2);
    assert_eq!(got, (4u64, 5u128, true));
    assert_eq!(0, result);

    let call_data =
        setFooEnumVariantCCall::new((bar_struct_id, Numbers::Two, Colors::Blue)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = getFooEnumVariantCCall::new((bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getFooEnumVariantCCall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1);
    assert_eq!(got, (Numbers::Two, Colors::Blue));
    assert_eq!(0, result);

    let call_data = getAddressCall::new((bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getAddressCall::abi_decode_returns(&return_data).unwrap();
    let got = return_data;
    assert_eq!(got, address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"));
    assert_eq!(0, result);

    let storage_before_destroy = runtime.get_storage();
    let call_data = destroyBarStructCall::new((bar_struct_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    let storage_after_destroy = runtime.get_storage();

    // Assert that the storage is empty
    assert_empty_storage(&storage_before_destroy, &storage_after_destroy);
}

#[rstest]
fn test_generic_bar_struct(
    #[with("enums", "tests/storage/move_sources/enums.move")] runtime: RuntimeSandbox,
) {
    runtime.set_msg_sender(SIGNER_ADDRESS);

    let call_data = createGenericBarStructCall::new((SIGNER_ADDRESS.into(),)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let generic_bar_struct_id = runtime.obtain_uid();

    let call_data = getGenericFooEnumVariantBCall::new((generic_bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getGenericFooEnumVariantBCall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1, return_data._2);
    assert_eq!(got, (42u64, 43u128, true));
    assert_eq!(0, result);

    let call_data = setGenericFooEnumVariantACall::new((generic_bar_struct_id, 2, 3)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = getGenericFooEnumVariantACall::new((generic_bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getGenericFooEnumVariantACall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1);
    assert_eq!(got, (2u16, 3u32));
    assert_eq!(0, result);

    let call_data =
        setGenericFooEnumVariantBCall::new((generic_bar_struct_id, 4, 5, true)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = getGenericFooEnumVariantBCall::new((generic_bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getGenericFooEnumVariantBCall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1, return_data._2);
    assert_eq!(got, (4u64, 5u128, true));
    assert_eq!(0, result);

    let call_data =
        setGenericFooEnumVariantCCall::new((generic_bar_struct_id, Numbers::Two, Colors::Blue))
            .abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);

    let call_data = getGenericFooEnumVariantCCall::new((generic_bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getGenericFooEnumVariantCCall::abi_decode_returns(&return_data).unwrap();
    let got = (return_data._0, return_data._1);
    assert_eq!(got, (Numbers::Two, Colors::Blue));
    assert_eq!(0, result);

    let call_data = getGenericAddressCall::new((generic_bar_struct_id,)).abi_encode();
    let (result, return_data) = runtime.call_entrypoint(call_data).unwrap();
    let return_data = getGenericAddressCall::abi_decode_returns(&return_data).unwrap();
    let got = return_data;
    assert_eq!(got, address!("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"));
    assert_eq!(0, result);

    let storage_before_destroy = runtime.get_storage();
    let call_data = destroyGenericBarStructCall::new((generic_bar_struct_id,)).abi_encode();
    let (result, _) = runtime.call_entrypoint(call_data).unwrap();
    assert_eq!(0, result);
    let storage_after_destroy = runtime.get_storage();

    // Assert that the storage is empty
    assert_empty_storage(&storage_before_destroy, &storage_after_destroy);
}
