use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "cast_uint_128",
    "tests/operations_cast/move_sources/uint_128.move"
);

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
