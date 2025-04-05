use alloy::{dyn_abi::SolType, sol, sol_types::SolCall};
use common::{ModuleData, setup_wasmtime_module, translate_test_package};
use walrus::Module;

mod common;

const SOURCE_PATH: &str = "tests/primitives";

fn run_test(translated_package: &mut Module, call_data: Vec<u8>, expected_result: Vec<u8>) {
    let data = ModuleData {
        data: call_data,
        return_data: vec![],
    };
    let data_len = data.data.len() as i32;

    let (_, mut store, entrypoint) = setup_wasmtime_module(translated_package, data);

    let result = entrypoint.call(&mut store, data_len).unwrap();
    assert_eq!(result, 0);

    let store_data = store.data();

    assert_eq!(store_data.return_data, expected_result);
}

#[test]
fn test_uint_8() {
    const MODULE_NAME: &str = "uint_8";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint8);
        function getLocal(uint8 _z) external returns (uint8);
        function echo(uint8 x) external returns (uint8);
        function echo2(uint8 x, uint8 y) external returns (uint8);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(88,));
    run_test(&mut translated_package, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(50,));
    run_test(&mut translated_package, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(222,));
    run_test(&mut translated_package, data, expected_result);

    // Echo max uint8
    let data = echoCall::abi_encode(&echoCall::new((u8::MAX,)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(u8::MAX,));
    run_test(&mut translated_package, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint8,))>::abi_encode_params(&(222,));
    run_test(&mut translated_package, data, expected_result);
}

#[test]
fn test_uint_16() {
    const MODULE_NAME: &str = "uint_16";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint16);
        function getLocal(uint16 _z) external returns (uint16);
        function echo(uint16 x) external returns (uint16);
        function echo2(uint16 x, uint16 y) external returns (uint16);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(1616,));
    run_test(&mut translated_package, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(50,));
    run_test(&mut translated_package, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(222,));
    run_test(&mut translated_package, data, expected_result);

    // Echo max uint16
    let data = echoCall::abi_encode(&echoCall::new((u16::MAX,)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(u16::MAX,));
    run_test(&mut translated_package, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint16,))>::abi_encode_params(&(222,));
    run_test(&mut translated_package, data, expected_result);
}

#[test]
fn test_uint_32() {
    const MODULE_NAME: &str = "uint_32";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint32);
        function getLocal(uint32 _z) external returns (uint32);
        function echo(uint32 x) external returns (uint32);
        function echo2(uint32 x, uint32 y) external returns (uint32);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(3232,));
    run_test(&mut translated_package, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(50,));
    run_test(&mut translated_package, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(222,));
    run_test(&mut translated_package, data, expected_result);

    // Echo max uint32
    let data = echoCall::abi_encode(&echoCall::new((u32::MAX,)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(u32::MAX,));
    run_test(&mut translated_package, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint32,))>::abi_encode_params(&(222,));
    run_test(&mut translated_package, data, expected_result);
}

#[test]
fn test_uint_64() {
    const MODULE_NAME: &str = "uint_64";

    sol!(
        #[allow(missing_docs)]
        function getConstant() external returns (uint64);
        function getLocal(uint64 _z) external returns (uint64);
        function echo(uint64 x) external returns (uint64);
        function echo2(uint64 x, uint64 y) external returns (uint64);
    );

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

    let data = getConstantCall::abi_encode(&getConstantCall::new(()));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(6464,));
    run_test(&mut translated_package, data, expected_result);

    let data = getLocalCall::abi_encode(&getLocalCall::new((111,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(50,));
    run_test(&mut translated_package, data, expected_result);

    let data = echoCall::abi_encode(&echoCall::new((222,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(222,));
    run_test(&mut translated_package, data, expected_result);

    // Echo max uint64
    let data = echoCall::abi_encode(&echoCall::new((u64::MAX,)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(u64::MAX,));
    run_test(&mut translated_package, data, expected_result);

    let data = echo2Call::abi_encode(&echo2Call::new((111, 222)));
    let expected_result = <sol!((uint64,))>::abi_encode_params(&(222,));
    run_test(&mut translated_package, data, expected_result);
}
