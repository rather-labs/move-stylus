use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use anyhow::Result;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};
use rstest::{fixture, rstest};

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

mod uint_8 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function castDown(uint16 x) external returns (uint8);
        function castFromU128(uint128 x) external returns (uint8);
        function castFromU256(uint256 x) external returns (uint8);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_8";
        const SOURCE_PATH: &str = "tests/operations-cast/uint_8.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(castDownCall::new((250,)), 250)]
    #[case(castDownCall::new((u8::MAX as u16,)), u8::MAX)]
    #[case(castFromU128Call::new((8,)), 8)]
    #[case(castFromU128Call::new((u8::MAX as u128,)), u8::MAX)]
    #[case(castFromU256Call::new((U256::from(8),)), 8)]
    #[case(castFromU256Call::new((U256::from(u8::MAX),)), u8::MAX)]
    fn test_uint_8_cast<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u8,
    ) {
        let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(castDownCall::new((u8::MAX as u16 + 1,)))]
    #[case(castFromU128Call::new((u8::MAX as u128 + 1,)))]
    #[case(castFromU256Call::new((U256::from(u8::MAX) + U256::from(1),)))]
    fn test_uint_8_cast_overflow<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
    ) {
        run_test(runtime, call_data.abi_encode(), vec![])
            .expect_err("should fail")
            .to_string()
            .contains("wasm trap: wasm `unreachable` instruction executed");
    }
}

mod uint_16 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function castDown(uint32 x) external returns (uint16);
        function castUp(uint8 x) external returns (uint16);
        function castFromU128(uint128 x) external returns (uint16);
        function castFromU256(uint256 x) external returns (uint16);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_16";
        const SOURCE_PATH: &str = "tests/operations-cast/uint_16.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(castDownCall::new((3232,)), 3232)]
    #[case(castDownCall::new((u16::MAX as u32,)), u16::MAX)]
    #[case(castUpCall::new((8,)), 8)]
    #[case(castUpCall::new((u8::MAX,)), u8::MAX as u16)]
    #[case(castFromU128Call::new((1616,)), 1616)]
    #[case(castFromU128Call::new((u16::MAX as u128,)), u16::MAX)]
    #[case(castFromU256Call::new((U256::from(1616),)), 1616)]
    #[case(castFromU256Call::new((U256::from(u16::MAX),)), u16::MAX)]
    fn test_uint_16_cast<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u16,
    ) {
        let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(castDownCall::new((u16::MAX as u32 + 1,)))]
    #[case(castFromU128Call::new((u16::MAX as u128 + 1,)))]
    #[case(castFromU256Call::new((U256::from(u16::MAX) + U256::from(1),)))]
    fn test_uint_16_cast_overflow<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
    ) {
        run_test(runtime, call_data.abi_encode(), vec![])
            .expect_err("should fail")
            .to_string()
            .contains("wasm trap: wasm `unreachable` instruction executed");
    }
}

mod uint_32 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function castDown(uint64 x) external returns (uint32);
        function castUp(uint16 x) external returns (uint32);
        function castFromU128(uint128 x) external returns (uint32);
        function castFromU256(uint256 x) external returns (uint32);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_32";
        const SOURCE_PATH: &str = "tests/operations-cast/uint_32.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(castDownCall::new((6464,)), 6464)]
    #[case(castDownCall::new((u32::MAX as u64,)), u32::MAX)]
    #[case(castUpCall::new((1616,)), 1616)]
    #[case(castUpCall::new((u16::MAX,)), u16::MAX as u32)]
    #[case(castFromU128Call::new((3232,)), 3232)]
    #[case(castFromU128Call::new((u32::MAX as u128,)), u32::MAX)]
    #[case(castFromU256Call::new((U256::from(3232),)), 3232)]
    #[case(castFromU256Call::new((U256::from(u32::MAX),)), u32::MAX)]
    fn test_uint_32_cast<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u32,
    ) {
        let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(castDownCall::new((u32::MAX as u64 + 1,)))]
    #[case(castFromU128Call::new((u32::MAX as u128 + 1,)))]
    #[case(castFromU256Call::new((U256::from(u32::MAX) + U256::from(1),)))]
    fn test_uint_32_cast_overflow<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
    ) {
        run_test(runtime, call_data.abi_encode(), vec![])
            .expect_err("should fail")
            .to_string()
            .contains("wasm trap: wasm `unreachable` instruction executed");
    }
}

mod uint_64 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function castUp(uint32 x) external returns (uint64);
        function castFromU128(uint128 x) external returns (uint64);
        function castFromU256(uint256 x) external returns (uint64);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_64";
        const SOURCE_PATH: &str = "tests/operations-cast/uint_64.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(castUpCall::new((3232,)), 3232)]
    #[case(castUpCall::new((u32::MAX,)), u32::MAX as u64)]
    #[case(castFromU128Call::new((6464,)), 6464)]
    #[case(castFromU128Call::new((u64::MAX as u128,)), u64::MAX)]
    #[case(castFromU256Call::new((U256::from(6464),)), 6464)]
    #[case(castFromU256Call::new((U256::from(u64::MAX),)), u64::MAX)]
    fn test_uint_64_cast<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u64,
    ) {
        let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(castFromU128Call::new((u64::MAX as u128 + 1,)))]
    #[case(castFromU256Call::new((U256::from(u64::MAX) + U256::from(1),)))]
    fn test_uint_64_cast_overflow<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
    ) {
        run_test(runtime, call_data.abi_encode(), vec![])
            .expect_err("should fail")
            .to_string()
            .contains("wasm trap: wasm `unreachable` instruction executed");
    }
}

mod uint_128 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function castUp(uint16 x) external returns (uint128);
        function castUpU64(uint64 x) external returns (uint128);
        function castFromU256(uint256 x) external returns (uint128);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_128";
        const SOURCE_PATH: &str = "tests/operations-cast/uint_128.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(castUpCall::new((3232,)), 3232)]
    #[case(castUpCall::new((u16::MAX,)), u16::MAX as u128)]
    #[case(castUpU64Call::new((128128,)), 128128)]
    #[case(castUpU64Call::new((u64::MAX,)), u64::MAX as u128)]
    #[case(castFromU256Call::new((U256::from(128128),)), 128128)]
    #[case(castFromU256Call::new((U256::from(u128::MAX),)), u128::MAX)]
    fn test_uint_128_cast<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: u128,
    ) {
        let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }

    #[rstest]
    #[case(castFromU256Call::new((U256::from(u128::MAX) + U256::from(1),)))]
    fn test_uint_128_cast_overflow<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
    ) {
        run_test(runtime, call_data.abi_encode(), vec![])
            .expect_err("should fail")
            .to_string()
            .contains("wasm trap: wasm `unreachable` instruction executed");
    }
}

mod uint_256 {
    use super::*;

    sol!(
        #[allow(missing_docs)]
        function castUp(uint16 x) external returns (uint256);
        function castUpU64(uint64 x) external returns (uint256);
        function castUpU128(uint128 x) external returns (uint256);
    );

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "uint_256";
        const SOURCE_PATH: &str = "tests/operations-cast/uint_256.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    #[rstest]
    #[case(castUpCall::new((3232,)), U256::from(3232))]
    #[case(castUpCall::new((u16::MAX,)), U256::from(u16::MAX))]
    #[case(castUpU64Call::new((6464,)), U256::from(6464))]
    #[case(castUpU64Call::new((u64::MAX,)), U256::from(u64::MAX))]
    #[case(castUpU128Call::new((128128,)), U256::from(128128))]
    #[case(castUpU128Call::new((u128::MAX,)), U256::from(u128::MAX))]
    fn test_uint_128_cast<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: U256,
    ) {
        let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
        run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
    }
}
