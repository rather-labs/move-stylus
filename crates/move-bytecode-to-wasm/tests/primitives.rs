use alloy::{
    dyn_abi::SolType,
    hex::FromHex,
    primitives::{Address, U256},
    sol,
    sol_types::SolCall,
};
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) {
    let (result, return_data) = runtime.call_entrypoint(call_data);
    assert_eq!(result, 0);
    assert_eq!(return_data, expected_result);
}

#[test]
fn test_bool() {
    const MODULE_NAME: &str = "bool_type";
    const SOURCE_PATH: &str = "tests/primitives/bool.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (bool);
        function getLocal(bool _z) external returns (bool);
        function getCopiedLocal() external returns (bool, bool);
        function echo(bool x) external returns (bool);
        function echo2(bool x, bool y) external returns (bool);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(true,));
    run_test(&runtime, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((true,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result);

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((bool, bool))>::abi_encode_params(&(true, false));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((true,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(true,));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((false,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((true, false)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result);
}

#[test]
fn test_address() {
    const MODULE_NAME: &str = "address_type";
    const SOURCE_PATH: &str = "tests/primitives/address.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (address);
        function getLocal(address _z) external returns (address);
        function getCopiedLocal() external returns (address, address);
        function echo(address x) external returns (address);
        function echo2(address x, address y) external returns (address);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000000000001",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((Address::from_hex(
        "0x0000000000000000000000000000000000000022",
    )
    .unwrap(),)));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000000000011",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result);

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((address, address))>::abi_encode_params(&(
        Address::from_hex("0x0000000000000000000000000000000000000001").unwrap(),
        Address::from_hex("0x0000000000000000000000000000000000000022").unwrap(),
    ));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((Address::from_hex(
        "0x0000000000000000000000000000000000000033",
    )
    .unwrap(),)));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000000000033",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((Address::from_hex(
        "0x0000000000000000000000000000000000000044",
    )
    .unwrap(),)));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000000000044",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((
        Address::from_hex("0x0000000000000000000000000000000000000055").unwrap(),
        Address::from_hex("0x0000000000000000000000000000000000000066").unwrap(),
    )));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000000000066",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result);
}

#[test]
fn test_uint_8() {
    const MODULE_NAME: &str = "uint_8";
    const SOURCE_PATH: &str = "tests/primitives/uint_8.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint8);
        function getLocal(uint8 _z) external returns (uint8);
        function getCopiedLocal() external returns (uint8, uint8);
        function echo(uint8 x) external returns (uint8);
        function echo2(uint8 x, uint8 y) external returns (uint8);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(88,));
    run_test(&runtime, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result);

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint8, uint8))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);

    // Echo max uint8
    let data = echoCall::abi_encode(&echoCall::new((u8::MAX,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(u8::MAX,));
    run_test(&runtime, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);
}

#[test]
fn test_uint_16() {
    const MODULE_NAME: &str = "uint_16";
    const SOURCE_PATH: &str = "tests/primitives/uint_16.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint16);
        function getLocal(uint16 _z) external returns (uint16);
        function getCopiedLocal() external returns (uint16, uint16);
        function echo(uint16 x) external returns (uint16);
        function echo2(uint16 x, uint16 y) external returns (uint16);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(1616,));
    run_test(&runtime, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result);

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint16, uint16))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);

    // Echo max uint16
    let data = echoCall::abi_encode(&echoCall::new((u16::MAX,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(u16::MAX,));
    run_test(&runtime, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);
}

#[test]
fn test_uint_32() {
    const MODULE_NAME: &str = "uint_32";
    const SOURCE_PATH: &str = "tests/primitives/uint_32.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint32);
        function getLocal(uint32 _z) external returns (uint32);
        function getCopiedLocal() external returns (uint32, uint32);
        function echo(uint32 x) external returns (uint32);
        function echo2(uint32 x, uint32 y) external returns (uint32);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(3232,));
    run_test(&runtime, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result);

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint32, uint32))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);

    // Echo max uint32
    let data = echoCall::abi_encode(&echoCall::new((u32::MAX,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(u32::MAX,));
    run_test(&runtime, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);
}

#[test]
fn test_uint_64() {
    const MODULE_NAME: &str = "uint_64";
    const SOURCE_PATH: &str = "tests/primitives/uint_64.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint64);
        function getLocal(uint64 _z) external returns (uint64);
        function getCopiedLocal() external returns (uint64, uint64);
        function echo(uint64 x) external returns (uint64);
        function echo2(uint64 x, uint64 y) external returns (uint64);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(6464,));
    run_test(&runtime, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result);

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint64, uint64))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);

    // Echo max uint64
    let data = echoCall::abi_encode(&echoCall::new((u64::MAX,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(u64::MAX,));
    run_test(&runtime, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);
}

#[test]
fn test_uint_128() {
    const MODULE_NAME: &str = "uint_128";
    const SOURCE_PATH: &str = "tests/primitives/uint_128.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint128);
        function getLocal(uint128 _z) external returns (uint128);
        function getCopiedLocal() external returns (uint128, uint128);
        function echo(uint128 x) external returns (uint128);
        function echo2(uint128 x, uint128 y) external returns (uint128);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(128128,));
    run_test(&runtime, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result);

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint128, uint128))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);

    // Echo max uint128
    let data = echoCall::abi_encode(&echoCall::new((u128::MAX,)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(u128::MAX,));
    run_test(&runtime, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result);
}

#[test]
fn test_uint_256() {
    const MODULE_NAME: &str = "uint_256";
    const SOURCE_PATH: &str = "tests/primitives/uint_256.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint256);
        function getLocal(uint256 _z) external returns (uint256);
        function getCopiedLocal() external returns (uint256, uint256);
        function echo(uint256 x) external returns (uint256);
        function echo2(uint256 x, uint256 y) external returns (uint256);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(256256),));
    run_test(&runtime, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((U256::from(111),)));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(50),));
    run_test(&runtime, data, expected_result);

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result =
        <sol!((uint256, uint256))>::abi_encode_params(&(U256::from(100), U256::from(111)));
    run_test(&runtime, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((U256::from(222),)));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(222),));
    run_test(&runtime, data, expected_result);

    // Echo max uint256
    let data = echoCall::abi_encode(&echoCall::new((U256::MAX,)));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::MAX,));
    run_test(&runtime, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((U256::from(111), U256::from(222))));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(222),));
    run_test(&runtime, data, expected_result);
}

#[test]
fn test_multi_values_return() {
    const MODULE_NAME: &str = "multi_values_return";
    const SOURCE_PATH: &str = "tests/primitives/multi_values_return.move";

    sol!(
        #[allow(missing_docs)]
        function getConstants() external returns (uint256, uint64, uint32, uint8, bool, address, uint32[], uint128[]);
        function getConstantsReversed() external returns (uint128[], uint32[], address, bool, uint8, uint32, uint64, uint256);
        function getConstantsNested() external returns (uint256, uint64, uint32, uint8, bool, address, uint32[], uint128[]);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantsCall::abi_encode(&getConstantsCall::new(()));
    let expected_result = <sol!((
        uint256,
        uint64,
        uint32,
        uint8,
        bool,
        address,
        uint32[],
        uint128[]
    ))>::abi_encode_params(&(
        U256::from(256256),
        6464,
        3232,
        88,
        true,
        Address::from_hex("0x0000000000000000000000000000000000000001").unwrap(),
        vec![10, 20, 30],
        vec![100, 200, 300],
    ));
    run_test(&runtime, data, expected_result);

    let data = getConstantsReversedCall::abi_encode(&getConstantsReversedCall::new(()));
    let expected_result = <sol!((
        uint128[],
        uint32[],
        address,
        bool,
        uint8,
        uint32,
        uint64,
        uint256
    ))>::abi_encode_params(&(
        vec![100, 200, 300],
        vec![10, 20, 30],
        Address::from_hex("0x0000000000000000000000000000000000000001").unwrap(),
        true,
        88,
        3232,
        6464,
        U256::from(256256),
    ));
    run_test(&runtime, data, expected_result);

    let data = getConstantsNestedCall::abi_encode(&getConstantsNestedCall::new(()));
    let expected_result = <sol!((
        uint256,
        uint64,
        uint32,
        uint8,
        bool,
        address,
        uint32[],
        uint128[]
    ))>::abi_encode_params(&(
        U256::from(256256),
        6464,
        3232,
        88,
        true,
        Address::from_hex("0x0000000000000000000000000000000000000001").unwrap(),
        vec![10, 20, 30],
        vec![100, 200, 300],
    ));
    run_test(&runtime, data, expected_result);
}
