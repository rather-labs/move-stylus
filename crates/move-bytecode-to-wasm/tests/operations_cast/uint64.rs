use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "cast_uint_64",
    "tests/operations_cast/move_sources/uint_64.move"
);

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
