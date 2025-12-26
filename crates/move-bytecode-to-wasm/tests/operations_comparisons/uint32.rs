use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "comparisons_u32",
    "tests/operations_comparisons/move_sources/uint_32.move"
);

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
