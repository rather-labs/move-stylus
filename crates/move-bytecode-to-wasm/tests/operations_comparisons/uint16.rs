use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "comparisons_u16",
    "tests/operations_comparisons/move_sources/uint_16.move"
);

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
