use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "cast_uint_8",
    "tests/operations_cast/move_sources/uint_8.move"
);

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
fn test_uint_8_cast_overflow<T: SolCall>(#[by_ref] runtime: &RuntimeSandbox, #[case] call_data: T) {
    run_test(runtime, call_data.abi_encode(), vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
}
