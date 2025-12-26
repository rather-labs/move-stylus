use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "enums_magical_creatures",
    "tests/enums/move_sources/magical_creatures.move"
);

sol! {
    function testBeast(uint32 level, uint64 ferocity) external returns (uint32, uint64, uint32, uint64);
    function testGolem(uint32 level, uint128 density, uint64[] shards) external returns (uint32, uint64, uint32, uint64);
    function testSpirit(uint32 level, uint8[][] chants, uint64 age) external returns (uint32, uint64, uint32, uint64);
}

#[rstest]
#[case(testBeastCall::new((1u32, 2u64)), (1u32, 2u64, 2u32, 4u64))]
#[case(testBeastCall::new((3u32, 5u64)), (3u32, 15u64, 4u32, 20u64))]
#[case(testGolemCall::new((1u32, 10u128, vec![5u64, 7u64])), (1u32, 23u64, 2u32, 39u64))]
#[case(testGolemCall::new((2u32, 20u128, vec![3u64, 4u64, 6u64])), (2u32, 35u64, 3u32, 51u64))]
#[case(testSpiritCall::new((1u32, vec![vec![2u8, 3u8, 4u8]], 4u64)), (1u32, 8u64, 2u32, 15u64))]
#[case(testSpiritCall::new((2u32, vec![vec![1u8, 2u8], vec![3u8, 4u8, 5u8]], 6u64)), (2u32, 13u64, 3u32, 20u64))]
fn test_magical_creatures<T: SolCall, V: SolValue>(
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
