use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::{U256, address};
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "multi_values_return",
    "tests/primitives/move_sources/multi_values_return.move"
);

sol!(
    #[allow(missing_docs)]
    function getConstants() external returns (uint256, uint64, uint32, uint8, bool, address, uint32[], uint128[]);
    function getConstantsReversed() external returns (uint128[], uint32[], address, bool, uint8, uint32, uint64, uint256);
    function getConstantsNested() external returns (uint256, uint64, uint32, uint8, bool, address, uint32[], uint128[]);
);

#[rstest]
#[case(
        getConstantsCall::new(()),
        (
            U256::from(256256),
            6464,
            3232,
            88,
            true,
            address!("0x0000000000000000000000000000000000000001"),
            vec![10, 20, 30],
            vec![100, 200, 300],
        )
    )]
#[case(
        getConstantsReversedCall::new(()),
        (
            vec![100, 200, 300],
            vec![10, 20, 30],
            address!("0x0000000000000000000000000000000000000001"),
            true,
            88,
            3232,
            6464,
            U256::from(256256),
        )
    )]
#[case(
        getConstantsNestedCall::new(()),
        (
            U256::from(256256),
            6464,
            3232,
            88,
            true,
            address!("0x0000000000000000000000000000000000000001"),
            vec![10, 20, 30],
            vec![100, 200, 300],
        )
    )]
fn test_multi_values_return<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) where
    for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode_sequence(),
    )
    .unwrap();
}
