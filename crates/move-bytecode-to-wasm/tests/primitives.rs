use std::u128;

use alloy::{
    dyn_abi::SolType,
    hex::FromHex,
    primitives::{Address, U256},
    sol,
    sol_types::SolCall,
};
use anyhow::Result;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) -> Result<()> {
    let (result, return_data) = runtime.call_entrypoint(call_data)?;
    anyhow::ensure!(
        result == 0,
        "Function returned non-zero exit code: {result}"
    );
    anyhow::ensure!(return_data == expected_result, "return data mismatch");

    Ok(())
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
        function notTrue() external returns (bool);
        function not(bool x) external returns (bool);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(true,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getLocalCall::abi_encode(&getLocalCall::new((true,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((bool, bool))>::abi_encode_params(&(true, false));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((true,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(true,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((false,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echo2Call::abi_encode(&echo2Call::new((true, false)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = notTrueCall::abi_encode(&notTrueCall::new(()));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = notCall::abi_encode(&notCall::new((false,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(true,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = notCall::abi_encode(&notCall::new((true,)));
    let expected_result = <sol!((bool,))>::abi_encode_params(&(false,));
    run_test(&runtime, data, expected_result).unwrap();
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
    run_test(&runtime, data, expected_result).unwrap();

    let data = getLocalCall::abi_encode(&getLocalCall::new((Address::from_hex(
        "0x0000000000000000000000000000000000000022",
    )
    .unwrap(),)));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000000000011",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((address, address))>::abi_encode_params(&(
        Address::from_hex("0x0000000000000000000000000000000000000001").unwrap(),
        Address::from_hex("0x0000000000000000000000000000000000000022").unwrap(),
    ));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((Address::from_hex(
        "0x0000000000000000000000000000000000000033",
    )
    .unwrap(),)));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000000000033",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((Address::from_hex(
        "0x0000000000000000000000000000000000000044",
    )
    .unwrap(),)));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000000000044",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echo2Call::abi_encode(&echo2Call::new((
        Address::from_hex("0x0000000000000000000000000000000000000055").unwrap(),
        Address::from_hex("0x0000000000000000000000000000000000000066").unwrap(),
    )));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000000000066",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();
}

#[test]
fn test_signer() {
    const MODULE_NAME: &str = "signer_type";
    const SOURCE_PATH: &str = "tests/primitives/signer.move";

    sol!(
        #[allow(missing_docs)]
        function echo() external returns (address);
        function echoIdentity() external returns (address);
        function echoWithInt(uint8 y) external returns (uint8, address);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = echoCall::abi_encode(&echoCall::new(()));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000007030507",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = echoIdentityCall::abi_encode(&echoIdentityCall::new(()));
    let expected_result = <sol!((address,))>::abi_encode_params(&(Address::from_hex(
        "0x0000000000000000000000000000000007030507",
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = echoWithIntCall::abi_encode(&echoWithIntCall::new((42,)));
    let expected_result = <sol!((uint8, address))>::abi_encode_params(&(
        42,
        Address::from_hex("0x0000000000000000000000000000000007030507").unwrap(),
    ));
    run_test(&runtime, data, expected_result).unwrap();
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
        function sum(uint8 x, uint8 y) external returns (uint8);
        function castU8(uint16 x) external returns (uint8);
        function castU8FromU128(uint128 x) external returns (uint8);
        function castU8FromU256(uint256 x) external returns (uint8);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(88,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint8, uint8))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // Echo max uint8
    let data = echoCall::abi_encode(&echoCall::new((u8::MAX,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(u8::MAX,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // --- SUM ---
    let data = sumCall::abi_encode(&sumCall::new((42, 42)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(84,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = sumCall::abi_encode(&sumCall::new((u8::MAX, 1)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    // --- CAST ---
    let data = castU8Call::abi_encode(&castU8Call::new((250,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(250,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU8Call::abi_encode(&castU8Call::new((u8::MAX as u16 + 1,)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    let data = castU8FromU128Call::abi_encode(&castU8FromU128Call::new((8u128,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(8,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU8FromU128Call::abi_encode(&castU8FromU128Call::new((u8::MAX as u128 + 1,)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    let data = castU8FromU256Call::abi_encode(&castU8FromU256Call::new((U256::from(8),)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(8,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU8FromU256Call::abi_encode(&castU8FromU256Call::new((
        U256::from(u8::MAX) + U256::from(1),
    )));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
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
        function sum(uint16 x, uint16 y) external returns (uint16);
        function castU16Down(uint32 x) external returns (uint16);
        function castU16Up(uint8 x) external returns (uint16);
        function castU16FromU128(uint128 x) external returns (uint16);
        function castU16FromU256(uint256 x) external returns (uint16);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(1616,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint16, uint16))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // Echo max uint16
    let data = echoCall::abi_encode(&echoCall::new((u16::MAX,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(u16::MAX,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // --- SUM ---
    let data = sumCall::abi_encode(&sumCall::new((255, 255)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(510,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = sumCall::abi_encode(&sumCall::new((u16::MAX, 1)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    // --- CAST ---
    let data = castU16DownCall::abi_encode(&castU16DownCall::new((1616,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(1616,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU16UpCall::abi_encode(&castU16UpCall::new((250,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(250,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU16DownCall::abi_encode(&castU16DownCall::new((u16::MAX as u32 + 1,)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    let data = castU16FromU128Call::abi_encode(&castU16FromU128Call::new((1616u128,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(1616,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU16FromU128Call::abi_encode(&castU16FromU128Call::new((u16::MAX as u128 + 1,)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    let data = castU16FromU256Call::abi_encode(&castU16FromU256Call::new((U256::from(1616),)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(1616,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU16FromU256Call::abi_encode(&castU16FromU256Call::new((
        U256::from(u16::MAX) + U256::from(1),
    )));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
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
        function sum(uint32 x, uint32 y) external returns (uint32);
        function castU32Down(uint64 x) external returns (uint32);
        function castU32Up(uint16 x) external returns (uint32);
        function castU32FromU128(uint128 x) external returns (uint32);
        function castU32FromU256(uint256 x) external returns (uint32);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(3232,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint32, uint32))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // Echo max uint32
    let data = echoCall::abi_encode(&echoCall::new((u32::MAX,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(u32::MAX,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // --- SUM ---
    let data = sumCall::abi_encode(&sumCall::new((65535, 65535)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(131070,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = sumCall::abi_encode(&sumCall::new((u32::MAX, 1)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    // --- CAST ---
    let data = castU32DownCall::abi_encode(&castU32DownCall::new((3232,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(3232,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU32UpCall::abi_encode(&castU32UpCall::new((250,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(250,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU32DownCall::abi_encode(&castU32DownCall::new((u32::MAX as u64 + 1,)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    let data = castU32FromU128Call::abi_encode(&castU32FromU128Call::new((3232u128,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(3232,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU32FromU128Call::abi_encode(&castU32FromU128Call::new((u32::MAX as u128 + 1,)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    let data = castU32FromU256Call::abi_encode(&castU32FromU256Call::new((U256::from(3232),)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(3232,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU32FromU256Call::abi_encode(&castU32FromU256Call::new((
        U256::from(u32::MAX) + U256::from(1),
    )));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
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
        function sum(uint64 x, uint64 y) external returns (uint64);
        function castU64Up(uint32 x) external returns (uint64);
        function castU64FromU128(uint128 x) external returns (uint64);
        function castU64FromU256(uint256 x) external returns (uint64);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(6464,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint64, uint64))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // Echo max uint64
    let data = echoCall::abi_encode(&echoCall::new((u64::MAX,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(u64::MAX,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // --- SUM ---
    let data = sumCall::abi_encode(&sumCall::new((4294967295, 4294967295)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(8589934590,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = sumCall::abi_encode(&sumCall::new((u64::MAX, 1)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    // --- CAST ---
    let data = castU64UpCall::abi_encode(&castU64UpCall::new((250,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(250,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU64FromU128Call::abi_encode(&castU64FromU128Call::new((6464u128,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(6464,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU64FromU128Call::abi_encode(&castU64FromU128Call::new((u64::MAX as u128 + 1,)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");

    let data = castU64FromU256Call::abi_encode(&castU64FromU256Call::new((U256::from(6464),)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(6464,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU64FromU256Call::abi_encode(&castU64FromU256Call::new((
        U256::from(u64::MAX) + U256::from(1),
    )));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
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
        function castU128Up(uint16 x) external returns (uint128);
        function castU128UpU64(uint64 x) external returns (uint128);
        function castU128FromU256(uint256 x) external returns (uint128);
        function sum(uint128 x, uint128 y) external returns (uint128);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(128128,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(50,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint128, uint128))>::abi_encode_params(&(100, 111));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // Echo max uint128
    let data = echoCall::abi_encode(&echoCall::new((u128::MAX,)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(u128::MAX,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(222,));
    run_test(&runtime, data, expected_result).unwrap();

    // --- CAST ---
    let data = castU128UpCall::abi_encode(&castU128UpCall::new((3232,)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(3232,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU128UpU64Call::abi_encode(&castU128UpU64Call::new((128128,)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(128128,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU128FromU256Call::abi_encode(&castU128FromU256Call::new((U256::from(128128),)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(128128,));
    run_test(&runtime, data, expected_result).unwrap();

    let data =
        castU128FromU256Call::abi_encode(&castU128FromU256Call::new((U256::MAX + U256::from(1),)));
    // The following tests test two situations:
    // 1. What happens when there is carry: we process the sum in chunks of 32 bits, so we use
    //    numbers in the form 2^(n*32) where n=1,2,3,4.
    //    If we add two numbers 2^(n*64) - 1, wthe first 64 bits will overflow and we will have to
    //    take care of the carry.
    //
    //    For example
    //    2^64 - 1 = [0, ..., 0, 0, 255, 255, 255, 255]
    //
    // 2. What happens if there is not carry :
    //    If we add two numbers 2^(n*64), the first 64 bits will of both numbers will be zero, so,
    //    when we add them there will be no carry at the beginning.
    //
    //    For example
    //    2^64     = [0, ..., 0, 0, 1, 0, 0, 0, 0]
    //
    // This tests are repeated for all the 32 bits chunks in the 256bits so we test a big number
    // that does not overflows
    //
    // Test the first 64 bits
    let data = sumCall::abi_encode(&sumCall::new((1, 1)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(2,));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^32 - 1
    // Expected result is (2^32 - 1) * 2
    let data = sumCall::abi_encode(&sumCall::new((4294967295, 4294967295)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(8589934590,));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^32
    // Expected result is 2^32 * 2
    let data = sumCall::abi_encode(&sumCall::new((4294967296, 4294967296)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(8589934592,));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^64 - 1
    // Expected result is (2^64 - 1) * 2
    let data = sumCall::abi_encode(&sumCall::new((18446744073709551615, 18446744073709551615)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(36893488147419103230,));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^64
    // Expected result is (2^64) * 2
    let data = sumCall::abi_encode(&sumCall::new((18446744073709551616, 18446744073709551616)));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(36893488147419103232,));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^96 - 1
    // Expected result is (2^96 - 1) * 2
    let data = sumCall::abi_encode(&sumCall::new((
        79228162514264337593543950335,
        79228162514264337593543950335,
    )));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(158456325028528675187087900670,));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^96
    // Expected result is 2^96 * 2
    let data = sumCall::abi_encode(&sumCall::new((
        79228162514264337593543950336,
        79228162514264337593543950336,
    )));
    let expected_result = <sol!((uint128,))>::abi_encode_params(&(158456325028528675187087900672,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = sumCall::abi_encode(&sumCall::new((u128::MAX, 42)));
    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
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
        function castU256Up(uint16 x) external returns (uint256);
        function castU256UpU64(uint64 x) external returns (uint256);
        function castU256UpU128(uint128 x) external returns (uint256);
        function sum(uint256 x, uint256 y) external returns (uint256);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(256256),));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getLocalCall::abi_encode(&getLocalCall::new((U256::from(111),)));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(50),));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result =
        <sol!((uint256, uint256))>::abi_encode_params(&(U256::from(100), U256::from(111)));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echoCall::abi_encode(&echoCall::new((U256::from(222),)));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(222),));
    run_test(&runtime, data, expected_result).unwrap();

    // Echo max uint256
    let data = echoCall::abi_encode(&echoCall::new((U256::MAX,)));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::MAX,));
    run_test(&runtime, data, expected_result).unwrap();

    let data = echo2Call::abi_encode(&echo2Call::new((U256::from(111), U256::from(222))));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(222),));
    run_test(&runtime, data, expected_result).unwrap();

    // --- CAST ---
    let data = castU256UpCall::abi_encode(&castU256UpCall::new((3232,)));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(3232),));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU256UpU64Call::abi_encode(&castU256UpU64Call::new((128128,)));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(128128),));
    run_test(&runtime, data, expected_result).unwrap();

    let data = castU256UpU128Call::abi_encode(&castU256UpU128Call::new((u128::MAX,)));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(u128::MAX),));
    run_test(&runtime, data, expected_result).unwrap();
    // The following tests test two situations:
    // 1. What happens when there is carry: we process the sum in chunks of 32 bits, so we use
    //    numbers in the form 2^(n*32) where n=1,2,3,4,5,6,7,8.
    //    If we add two numbers 2^(n*64) - 1, wthe first 64 bits will overflow and we will have to
    //    take care of the carry.
    //
    //    For example
    //    2^64 - 1 = [0, ..., 0, 0, 255, 255, 255, 255]
    //
    // 2. What happens if there is not carry :
    //    If we add two numbers 2^(n*64), the first 64 bits will of both numbers will be zero, so,
    //    when we add them there will be no carry at the beginning.
    //
    //    For example
    //    2^64     = [0, ..., 0, 0, 1, 0, 0, 0, 0]
    //
    // This tests are repeated for all the 32 bits chunks in the 256bits so we test a big number
    // that does not overflows

    // Test the first 64 bits
    let data = sumCall::abi_encode(&sumCall::new((U256::from(1), U256::from(1))));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from(2),));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^32 - 1
    // Expected result is (2^32 - 1) * 2
    let data = sumCall::abi_encode(&sumCall::new((
        U256::from_str_radix("4294967295", 10).unwrap(),
        U256::from_str_radix("4294967295", 10).unwrap(),
    )));
    let expected_result =
        <sol!((uint256,))>::abi_encode_params(&(U256::from_str_radix("8589934590", 10).unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^32
    // Expected result is 2^32 * 2
    let data = sumCall::abi_encode(&sumCall::new((
        U256::from_str_radix("4294967296", 10).unwrap(),
        U256::from_str_radix("4294967296", 10).unwrap(),
    )));
    let expected_result =
        <sol!((uint256,))>::abi_encode_params(&(U256::from_str_radix("8589934592", 10).unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^64 - 1
    // Expected result is (2^64 - 1) * 2
    let data = sumCall::abi_encode(&sumCall::new((
        U256::from_str_radix("18446744073709551615", 10).unwrap(),
        U256::from_str_radix("18446744073709551615", 10).unwrap(),
    )));
    let expected_result =
        <sol!((uint256,))>::abi_encode_params(&(
            U256::from_str_radix("36893488147419103230", 10).unwrap(),
        ));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^64
    // Expected result is (2^64) * 2
    let data = sumCall::abi_encode(&sumCall::new((
        U256::from_str_radix("18446744073709551616", 10).unwrap(),
        U256::from_str_radix("18446744073709551616", 10).unwrap(),
    )));
    let expected_result =
        <sol!((uint256,))>::abi_encode_params(&(
            U256::from_str_radix("36893488147419103232", 10).unwrap(),
        ));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^96 - 1
    // Expected result is (2^96 - 1) * 2
    let data = sumCall::abi_encode(&sumCall::new((
        U256::from_str_radix("79228162514264337593543950335", 10).unwrap(),
        U256::from_str_radix("79228162514264337593543950335", 10).unwrap(),
    )));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from_str_radix(
        "158456325028528675187087900670",
        10,
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^96
    // Expected result is 2^96 * 2
    let data = sumCall::abi_encode(&sumCall::new((
        U256::from_str_radix("79228162514264337593543950336", 10).unwrap(),
        U256::from_str_radix("79228162514264337593543950336", 10).unwrap(),
    )));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from_str_radix(
        "158456325028528675187087900672",
        10,
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    // Operands are 2^128 - 1
    // Expected result is (2^128 - 1) * 2
    let data = sumCall::abi_encode(&sumCall::new((
        U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
        U256::from_str_radix("340282366920938463463374607431768211456", 10).unwrap(),
    )));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from_str_radix(
        "680564733841876926926749214863536422912",
        10,
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    // Test the carry to the third half of 64 bits
    // Operands are 2^128 - 1
    // Expected result is (2^128 - 1) * 2
    let data = sumCall::abi_encode(&sumCall::new((
        U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
        U256::from_str_radix("340282366920938463463374607431768211455", 10).unwrap(),
    )));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from_str_radix(
        "680564733841876926926749214863536422910",
        10,
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    // Test the carry to the fourth half of 64 bits
    // Operands are 2^192 - 1
    // Expected result is (2^192 - 1) * 2)
    let data = sumCall::abi_encode(&sumCall::new((
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512895",
            10,
        )
        .unwrap(),
        U256::from_str_radix(
            "6277101735386680763835789423207666416102355444464034512895",
            10,
        )
        .unwrap(),
    )));
    let expected_result = <sol!((uint256,))>::abi_encode_params(&(U256::from_str_radix(
        "12554203470773361527671578846415332832204710888928069025790",
        10,
    )
    .unwrap(),));
    run_test(&runtime, data, expected_result).unwrap();

    let data = sumCall::abi_encode(&sumCall::new((
        U256::MAX,
        U256::from_str_radix("42", 10).unwrap(),
    )));

    run_test(&runtime, data, vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
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
    run_test(&runtime, data, expected_result).unwrap();

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
    run_test(&runtime, data, expected_result).unwrap();

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
    run_test(&runtime, data, expected_result).unwrap();
}

#[test]
fn test_vec_32() {
    const MODULE_NAME: &str = "vec_32";
    const SOURCE_PATH: &str = "tests/primitives/vec_32.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint32[]);
        function getConstantLocal() external returns (uint32[]);
        function getLiteral() external returns (uint32[]);
        function getCopiedLocal() external returns (uint32[]);
        function echo(uint32[] x) external returns (uint32[]);
        function vecFromInt(uint32 x, uint32 y) external returns (uint32[]);
        function vecFromVec(uint32[] x, uint32[] y) external returns (uint32[][]);
        function vecFromVecAndInt(uint32[] x, uint32 y) external returns (uint32[][]);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantLocalCall::abi_encode(&getConstantLocalCall::new(()));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // getLiteral() should return [1, 2, 3]
    let data = getLiteralCall::abi_encode(&getLiteralCall::new(()));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // getCopiedLocal() should return [1, 2, 3]
    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // echo([1, 2, 3]) should return [1, 2, 3]
    let data = echoCall::abi_encode(&echoCall::new((vec![1u32, 2u32, 3u32],)));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 3u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromInt(1, 2) should return [1, 2]
    let data = vecFromIntCall::abi_encode(&vecFromIntCall::new((1u32, 2u32)));
    let expected_result = <sol!((uint32[],))>::abi_encode_params(&(vec![1u32, 2u32, 1u32],));
    run_test(&runtime, data, expected_result).unwrap();

    // vec_from_vec([1, 2, 3], [4, 5, 6]) should return [[1, 2, 3], [4, 5, 6]]
    let data = vecFromVecCall::abi_encode(&vecFromVecCall::new((
        vec![1u32, 2u32, 3u32],
        vec![4u32, 5u32, 6u32],
    )));
    let expected_result = <sol!((uint32[][],))>::abi_encode_params(&(vec![
        vec![1u32, 2u32, 3u32],
        vec![4u32, 5u32, 6u32],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromVecAndInt([1, 2, 3], 4) should return [[1, 2, 3], [4, 4]]
    let data = vecFromVecAndIntCall::abi_encode(&vecFromVecAndIntCall::new((
        vec![1u32, 2u32, 3u32],
        4u32,
    )));
    let expected_result =
        <sol!((uint32[][],))>::abi_encode_params(
            &(vec![vec![1u32, 2u32, 3u32], vec![4u32, 4u32]],),
        );
    run_test(&runtime, data, expected_result).unwrap();
}

#[test]
fn test_vec_64() {
    const MODULE_NAME: &str = "vec_64";
    const SOURCE_PATH: &str = "tests/primitives/vec_64.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint64[]);
        function getConstantLocal() external returns (uint64[]);
        function getLiteral() external returns (uint64[]);
        function getCopiedLocal() external returns (uint64[]);
        function echo(uint64[] x) external returns (uint64[]);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantLocalCall::abi_encode(&getConstantLocalCall::new(()));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();

    // getLiteral() should return [1, 2, 3]
    let data = getLiteralCall::abi_encode(&getLiteralCall::new(()));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();

    // getCopiedLocal() should return [1, 2, 3]
    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();

    // echo([1, 2, 3]) should return [1, 2, 3]
    let data = echoCall::abi_encode(&echoCall::new((vec![1u64, 2u64, 3u64],)));
    let expected_result = <sol!((uint64[],))>::abi_encode_params(&(vec![1u64, 2u64, 3u64],));
    run_test(&runtime, data, expected_result).unwrap();
}

#[test]
fn test_vec_128() {
    const MODULE_NAME: &str = "vec_128";
    const SOURCE_PATH: &str = "tests/primitives/vec_128.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint128[]);
        function getConstantLocal() external returns (uint128[]);
        function getLiteral() external returns (uint128[]);
        function getCopiedLocal() external returns (uint128[]);
        function echo(uint128[] x) external returns (uint128[]);
        function vecFromInt(uint128 x, uint128 y) external returns (uint128[]);
        function vecFromVec(uint128[] x, uint128[] y) external returns (uint128[][]);
        function vecFromVecAndInt(uint128[] x, uint128 y) external returns (uint128[][]);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantLocalCall::abi_encode(&getConstantLocalCall::new(()));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // getLiteral() should return [1, 2, 3]
    let data = getLiteralCall::abi_encode(&getLiteralCall::new(()));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // getCopiedLocal() should return [1, 2, 3]
    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // echo([1, 2, 3]) should return [1, 2, 3]
    let data = echoCall::abi_encode(&echoCall::new((vec![1u128, 2u128, 3u128],)));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 3u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromInt(1, 2) should return [1, 2, 1]
    let data = vecFromIntCall::abi_encode(&vecFromIntCall::new((1u128, 2u128)));
    let expected_result = <sol!((uint128[],))>::abi_encode_params(&(vec![1u128, 2u128, 1u128],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromVec([1, 2, 3], [4, 5, 6]) should return [[1, 2, 3], [4, 5, 6]]
    let data = vecFromVecCall::abi_encode(&vecFromVecCall::new((
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
    )));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    // vecFromVecAndInt([1, 2, 3], 4) should return [[1, 2, 3], [4, 4]]
    let data = vecFromVecAndIntCall::abi_encode(&vecFromVecAndIntCall::new((
        vec![1u128, 2u128, 3u128],
        4u128,
    )));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 4u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();
}

#[test]
fn test_vec_vec_128() {
    const MODULE_NAME: &str = "vec_vec_128";
    const SOURCE_PATH: &str = "tests/primitives/vec_vec_128.move";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint128[][]);
        function getConstantLocal() external returns (uint128[][]);
        function getLiteral() external returns (uint128[][]);
        function getCopiedLocal() external returns (uint128[][]);
        function echo(uint128[][] x) external returns (uint128[][]);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
    let runtime = RuntimeSandbox::new(&mut translated_package);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    let data = getConstantLocalCall::abi_encode(&getConstantLocalCall::new(()));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();
    // getLiteral() should return [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
    let data = getLiteralCall::abi_encode(&getLiteralCall::new(()));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    // getCopiedLocal() should return [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
    let data = getCopiedLocalCall::abi_encode(&getCopiedLocalCall::new(()));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();

    // echo([[1, 2, 3], [4, 5, 6], [7, 8, 9]]) should return the same
    let data = echoCall::abi_encode(&echoCall::new((vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],)));
    let expected_result = <sol!((uint128[][],))>::abi_encode_params(&(vec![
        vec![1u128, 2u128, 3u128],
        vec![4u128, 5u128, 6u128],
        vec![7u128, 8u128, 9u128],
    ],));
    run_test(&runtime, data, expected_result).unwrap();
}
