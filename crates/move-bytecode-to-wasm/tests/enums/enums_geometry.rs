use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("enums_geometry", "tests/enums/move_sources/geometry.move");

sol! {
    function testSquare(uint64 side) external returns (uint64, uint64);
    function testMutateSquare(uint64 side) external returns (uint64, uint64, uint64, uint64);
    function testTriangle(uint64 base, uint64 height) external returns (uint64, uint64, uint64);
    function testMutateTriangle(uint64 base, uint64 height) external returns (uint64, uint64, uint64, uint64, uint64, uint64);
    function testVectorOfShapes1(uint64 a, uint64 b) external returns (uint64, uint64, uint64);
    function testVectorOfShapes2(uint64 a, uint64 b) external returns (uint64, uint64, uint64);
}

#[rstest]
#[case(testSquareCall::new((4u64,)), (4u64, 16u64))]
#[case(testSquareCall::new((5u64,)), (5u64, 25u64))]
#[case(testMutateSquareCall::new((4u64,)), (4u64, 16u64, 5u64, 25u64))]
#[case(testMutateSquareCall::new((5u64,)), (5u64, 25u64, 6u64, 36u64))]
#[case(testTriangleCall::new((4u64, 5u64)), (4u64, 5u64, 10u64))]
#[case(testTriangleCall::new((5u64, 6u64)), (5u64, 6u64, 15u64))]
#[case(testMutateTriangleCall::new((4u64, 5u64)), (4u64, 5u64, 10u64, 5u64, 6u64, 15u64))]
#[case(testMutateTriangleCall::new((5u64, 6u64)), (5u64, 6u64, 15u64, 6u64, 7u64, 21u64))]
#[case(testVectorOfShapes1Call::new((2u64, 3u64)), (2u64, 4u64, 3u64))]
#[case(testVectorOfShapes2Call::new((2u64, 3u64)), (2u64, 3u64, 4u64))]
fn test_geometry<T: SolCall, V: SolValue>(
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
