use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "comparisons_u128",
    "tests/operations_comparisons/move_sources/uint_128.move"
);

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
