use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "comparisons_u256",
    "tests/operations_comparisons/move_sources/uint_256.move"
);

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
