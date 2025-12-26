use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "comparisons_u8",
    "tests/operations_comparisons/move_sources/uint_8.move"
);

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
