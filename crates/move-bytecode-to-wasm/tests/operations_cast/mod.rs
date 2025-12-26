use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

mod uint_8 {
    use super::*;

    declare_fixture!("cast_uint_8", "tests/operations_cast/uint_8.move");

    sol!(
        #[allow(missing_docs)]
        function castDown(uint16 x) external returns (uint8);
        function castFromU128(uint128 x) external returns (uint8);
        function castFromU256(uint256 x) external returns (uint8);
    );

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

    declare_fixture!("cast_uint_16", "tests/operations_cast/uint_16.move");

    sol!(
        #[allow(missing_docs)]
        function castDown(uint32 x) external returns (uint16);
        function castUp(uint8 x) external returns (uint16);
        function castFromU128(uint128 x) external returns (uint16);
        function castFromU256(uint256 x) external returns (uint16);
    );

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

    declare_fixture!("cast_uint_32", "tests/operations_cast/uint_32.move");

    sol!(
        #[allow(missing_docs)]
        function castDown(uint64 x) external returns (uint32);
        function castUp(uint16 x) external returns (uint32);
        function castFromU128(uint128 x) external returns (uint32);
        function castFromU256(uint256 x) external returns (uint32);
    );

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

    declare_fixture!("cast_uint_64", "tests/operations_cast/uint_64.move");

    sol!(
        #[allow(missing_docs)]
        function castUp(uint32 x) external returns (uint64);
        function castFromU128(uint128 x) external returns (uint64);
        function castFromU256(uint256 x) external returns (uint64);
    );

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

    declare_fixture!("cast_uint_128", "tests/operations_cast/uint_128.move");

    sol!(
        #[allow(missing_docs)]
        function castUp(uint16 x) external returns (uint128);
        function castUpU64(uint64 x) external returns (uint128);
        function castFromU256(uint256 x) external returns (uint128);
    );

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

    declare_fixture!("cast_uint_256", "tests/operations_cast/uint_256.move");

    sol!(
        #[allow(missing_docs)]
        function castUp(uint16 x) external returns (uint256);
        function castUpU64(uint64 x) external returns (uint256);
        function castUpU128(uint128 x) external returns (uint256);
    );

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
