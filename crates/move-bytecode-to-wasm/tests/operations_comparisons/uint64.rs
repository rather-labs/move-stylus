use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "comparisons_u64",
    "tests/operations_comparisons/move_sources/uint_64.move"
);

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
