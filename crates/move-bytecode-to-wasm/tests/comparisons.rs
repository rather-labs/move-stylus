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
    anyhow::ensure!(
        return_data == expected_result,
        "return data mismatch:\nreturned:{return_data:?}\nexpected:{expected_result:?}"
    );

    Ok(())
}

mod comparisons_u8 {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "comparisons_u8";
        const SOURCE_PATH: &str = "tests/operations-comparisons/uint_8.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function lessThanU8(uint8 x, uint8 y) external returns (bool);
        function lessThanEqU8(uint8 x, uint8 y) external returns (bool);
        function greaterThanU8(uint8 x, uint8 y) external returns (bool);
        function greaterEqThanU8(uint8 x, uint8 y) external returns (bool);
    );

    #[rstest]
    #[case(lessThanU8Call::new((u8::MAX, u8::MAX)), false)]
    #[case(lessThanU8Call::new((u8::MAX - 1, u8::MAX - 2)), false)]
    #[case(lessThanU8Call::new((u8::MAX - 1, u8::MAX)), true)]
    #[case(lessThanEqU8Call::new((u8::MAX, u8::MAX)), true)]
    #[case(lessThanEqU8Call::new((u8::MAX - 1, u8::MAX - 2)), false)]
    #[case(lessThanEqU8Call::new((u8::MAX - 1, u8::MAX)), true)]
    #[case(greaterThanU8Call::new((u8::MAX, u8::MAX)), false)]
    #[case(greaterThanU8Call::new((u8::MAX, u8::MAX - 1)), true)]
    #[case(greaterThanU8Call::new((u8::MAX - 1, u8::MAX)), false)]
    #[case(greaterEqThanU8Call::new((u8::MAX, u8::MAX)), true)]
    #[case(greaterEqThanU8Call::new((u8::MAX, u8::MAX - 1)), true)]
    #[case(greaterEqThanU8Call::new((u8::MAX - 1, u8::MAX)), false)]
    fn test_comparisons_u8<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((bool,))>::abi_encode(&(expected_result,)),
        )
        .unwrap();
    }
}

mod comparisons_u16 {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "comparisons_u16";
        const SOURCE_PATH: &str = "tests/operations-comparisons/uint_16.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function lessThanU16(uint16 x, uint16 y) external returns (bool);
        function lessThanEqU16(uint16 x, uint16 y) external returns (bool);
        function greaterThanU16(uint16 x, uint16 y) external returns (bool);
        function greaterEqThanU16(uint16 x, uint16 y) external returns (bool);
    );

    #[rstest]
    #[case(lessThanU16Call::new((u16::MAX, u16::MAX)), false)]
    #[case(lessThanU16Call::new((u16::MAX - 1, u16::MAX - 2)), false)]
    #[case(lessThanU16Call::new((u16::MAX - 1, u16::MAX)), true)]
    #[case(lessThanEqU16Call::new((u16::MAX, u16::MAX)), true)]
    #[case(lessThanEqU16Call::new((u16::MAX - 1, u16::MAX - 2)), false)]
    #[case(lessThanEqU16Call::new((u16::MAX - 1, u16::MAX)), true)]
    #[case(greaterThanU16Call::new((u16::MAX, u16::MAX)), false)]
    #[case(greaterThanU16Call::new((u16::MAX, u16::MAX - 1)), true)]
    #[case(greaterThanU16Call::new((u16::MAX - 1, u16::MAX)), false)]
    #[case(greaterEqThanU16Call::new((u16::MAX, u16::MAX)), true)]
    #[case(greaterEqThanU16Call::new((u16::MAX, u16::MAX - 1)), true)]
    #[case(greaterEqThanU16Call::new((u16::MAX - 1, u16::MAX)), false)]
    fn test_comparison_u16<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((bool,))>::abi_encode(&(expected_result,)),
        )
        .unwrap();
    }
}

mod comparisons_u32 {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "comparisons_u32";
        const SOURCE_PATH: &str = "tests/operations-comparisons/uint_32.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function lessThanU32(uint32 x, uint32 y) external returns (bool);
        function lessThanEqU32(uint32 x, uint32 y) external returns (bool);
        function greaterThanU32(uint32 x, uint32 y) external returns (bool);
        function greaterEqThanU32(uint32 x, uint32 y) external returns (bool);
    );

    #[rstest]
    #[case(lessThanU32Call::new((u32::MAX, u32::MAX)), false)]
    #[case(lessThanU32Call::new((u32::MAX - 1, u32::MAX - 2)), false)]
    #[case(lessThanU32Call::new((u32::MAX - 1, u32::MAX)), true)]
    #[case(lessThanEqU32Call::new((u32::MAX, u32::MAX)), true)]
    #[case(lessThanEqU32Call::new((u32::MAX - 1, u32::MAX - 2)), false)]
    #[case(lessThanEqU32Call::new((u32::MAX - 1, u32::MAX)), true)]
    #[case(greaterThanU32Call::new((u32::MAX, u32::MAX)), false)]
    #[case(greaterThanU32Call::new((u32::MAX, u32::MAX - 1)), true)]
    #[case(greaterThanU32Call::new((u32::MAX - 1, u32::MAX)), false)]
    #[case(greaterEqThanU32Call::new((u32::MAX, u32::MAX)), true)]
    #[case(greaterEqThanU32Call::new((u32::MAX, u32::MAX - 1)), true)]
    #[case(greaterEqThanU32Call::new((u32::MAX - 1, u32::MAX)), false)]
    fn test_comparisons_u32<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((bool,))>::abi_encode(&(expected_result,)),
        )
        .unwrap();
    }
}

mod comparisons_u64 {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "comparisons_u64";
        const SOURCE_PATH: &str = "tests/operations-comparisons/uint_64.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function lessThanU64(uint64 x, uint64 y) external returns (bool);
        function lessThanEqU64(uint64 x, uint64 y) external returns (bool);
        function greaterThanU64(uint64 x, uint64 y) external returns (bool);
        function greaterEqThanU64(uint64 x, uint64 y) external returns (bool);
    );

    #[rstest]
    #[case(lessThanU64Call::new((u64::MAX, u64::MAX)), false)]
    #[case(lessThanU64Call::new((u64::MAX - 1, u64::MAX - 2)), false)]
    #[case(lessThanU64Call::new((u64::MAX - 1, u64::MAX)), true)]
    #[case(lessThanEqU64Call::new((u64::MAX, u64::MAX)), true)]
    #[case(lessThanEqU64Call::new((u64::MAX - 1, u64::MAX - 2)), false)]
    #[case(lessThanEqU64Call::new((u64::MAX - 1, u64::MAX)), true)]
    #[case(greaterThanU64Call::new((u64::MAX, u64::MAX)), false)]
    #[case(greaterThanU64Call::new((u64::MAX, u64::MAX - 1)), true)]
    #[case(greaterThanU64Call::new((u64::MAX - 1, u64::MAX)), false)]
    #[case(greaterEqThanU64Call::new((u64::MAX, u64::MAX)), true)]
    #[case(greaterEqThanU64Call::new((u64::MAX, u64::MAX - 1)), true)]
    #[case(greaterEqThanU64Call::new((u64::MAX - 1, u64::MAX)), false)]
    fn test_comparisons_u64<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((bool,))>::abi_encode(&(expected_result,)),
        )
        .unwrap();
    }
}

mod comparisons_u128 {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "comparisons_u128";
        const SOURCE_PATH: &str = "tests/operations-comparisons/uint_128.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function lessThanU128(uint128 x, uint128 y) external returns (bool);
        function lessThanEqU128(uint128 x, uint128 y) external returns (bool);
        function greaterThanU128(uint128 x, uint128 y) external returns (bool);
        function greaterEqThanU128(uint128 x, uint128 y) external returns (bool);
    );

    #[rstest]
    #[case(lessThanU128Call::new((u128::MAX, u128::MAX)), false)]
    #[case(lessThanU128Call::new((u128::MAX - 1, u128::MAX - 2)), false)]
    #[case(lessThanU128Call::new((u128::MAX - 1, u128::MAX)), true)]
    #[case(lessThanEqU128Call::new((u128::MAX, u128::MAX)), true)]
    #[case(lessThanEqU128Call::new((u128::MAX - 1, u128::MAX - 2)), false)]
    #[case(lessThanEqU128Call::new((u128::MAX - 1, u128::MAX)), true)]
    #[case(greaterThanU128Call::new((u128::MAX, u128::MAX)), false)]
    #[case(greaterThanU128Call::new((u128::MAX, u128::MAX - 1)), true)]
    #[case(greaterThanU128Call::new((u128::MAX - 1, u128::MAX)), false)]
    #[case(greaterEqThanU128Call::new((u128::MAX, u128::MAX)), true)]
    #[case(greaterEqThanU128Call::new((u128::MAX, u128::MAX - 1)), true)]
    #[case(greaterEqThanU128Call::new((u128::MAX - 1, u128::MAX)), false)]
    fn test_comparisons_u128<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((bool,))>::abi_encode(&(expected_result,)),
        )
        .unwrap();
    }
}

mod comparisons_u256 {
    use super::*;

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        const MODULE_NAME: &str = "comparisons_u256";
        const SOURCE_PATH: &str = "tests/operations-comparisons/uint_256.move";

        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function lessThanU256(uint256 x, uint256 y) external returns (bool);
        function lessThanEqU256(uint256 x, uint256 y) external returns (bool);
        function greaterThanU256(uint256 x, uint256 y) external returns (bool);
        function greaterEqThanU256(uint256 x, uint256 y) external returns (bool);
    );

    #[rstest]
    #[case(lessThanU256Call::new((U256::MAX, U256::MAX)), false)]
    #[case(lessThanU256Call::new((U256::MAX - U256::from(1), U256::MAX - U256::from(2))), false)]
    #[case(lessThanU256Call::new((U256::MAX - U256::from(1), U256::MAX)), true)]
    #[case(lessThanEqU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(lessThanEqU256Call::new((U256::MAX - U256::from(1), U256::MAX - U256::from(2))), false)]
    #[case(lessThanEqU256Call::new((U256::MAX - U256::from(1), U256::MAX)), true)]
    #[case(greaterThanU256Call::new((U256::MAX, U256::MAX)), false)]
    #[case(greaterThanU256Call::new((U256::MAX, U256::MAX - U256::from(1))), true)]
    #[case(greaterThanU256Call::new((U256::MAX - U256::from(1), U256::MAX)), false)]
    #[case(greaterEqThanU256Call::new((U256::MAX, U256::MAX)), true)]
    #[case(greaterEqThanU256Call::new((U256::MAX, U256::MAX - U256::from(1))), true)]
    #[case(greaterEqThanU256Call::new((U256::MAX - U256::from(1), U256::MAX)), false)]
    fn test_comparisons_u256<T: SolCall>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: bool,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            <sol!((bool,))>::abi_encode(&(expected_result,)),
        )
        .unwrap();
    }
}
