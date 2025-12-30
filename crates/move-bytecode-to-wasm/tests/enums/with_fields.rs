use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "enum_with_fields",
    "tests/enums/move_sources/enum_with_fields.move"
);

sol! {
    function packUnpackPlanet(uint8 index) external returns (uint64, uint64);
    function packUnpackStackInts(uint8 x, uint16 y, uint32 z, uint64 w) external returns (uint8, uint16, uint32, uint64);
    function packUnpackHeapInts(uint128 x, uint256 y) external returns (uint128, uint256);
    function packUnpackPositionalVector(uint8 a, uint16 b, uint32 c, uint64 d) external returns (uint8[], uint16[], uint32[], uint64[]);
    function packUnpackNamedVectors(uint128 x, uint256 y) external returns (uint128[], uint256[]);
    function packUnpackPositionalNestedVectors(uint32 x, uint64 y) external returns (uint32[][], uint64[][]);
    function packUnpackAlpha(uint8 a, uint16 b, uint32 c, uint64 d) external returns (uint8, uint16, uint32, uint64);
    function packUnpackBeta(uint128 e, uint256 f) external returns (uint128, uint256);
    function packUnpackGamma(uint32[] a, bool[] b, uint128 c, uint256 d) external returns (uint32[], bool[], uint128, uint256);
    function getGammaVecSum(uint32[] a, bool[] b, uint128 c, uint256 d) external returns (uint32);
}

#[rstest]
#[case(packUnpackPlanetCall::new((0,)), (6371, 5972))]
#[case(packUnpackPlanetCall::new((1,)), (69911, 1898000))]
#[case(packUnpackPlanetCall::new((2,)), (3389, 641))]
#[case(packUnpackPlanetCall::new((3,)), (6051, 4868))]
#[case(packUnpackPlanetCall::new((4,)), (58232, 56800))]
fn test_pack_unpack_planet<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) where
    for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}

#[rstest]
#[case(packUnpackStackIntsCall::new((0, 0u16, 0u32, 0u64)), (0, 0u16, 0u32, 0u64))]
#[case(packUnpackStackIntsCall::new((1, 2u16, 3u32, 4u64)), (1, 2u16, 3u32, 4u64))]
#[case(packUnpackStackIntsCall::new((255, u16::MAX, u32::MAX, u64::MAX)), (255, u16::MAX, u32::MAX, u64::MAX))]
#[case(packUnpackHeapIntsCall::new((0u128, U256::from(0u128))), (0u128, U256::from(0u128)))]
#[case(packUnpackHeapIntsCall::new((1u128, U256::from(2u128))), (1u128, U256::from(2u128)))]
#[case(packUnpackHeapIntsCall::new((u128::MAX, U256::from(u128::MAX))), (u128::MAX, U256::from(u128::MAX)))]
fn test_pack_unpack_ints<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) where
    for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}

#[rstest]
#[case(
        packUnpackPositionalVectorCall::new((87u8, 42u16, 55u32, 71u64)),
        (
            vec![87, 88, 89],
            vec![42u16, 43u16, 44u16],
            vec![55u32, 56u32, 57u32],
            vec![71u64, 72u64, 73u64],
        )
    )]
#[case(packUnpackNamedVectorsCall::new((0u128, U256::from(0u128))), (vec![0u128, 1u128, 2u128], vec![U256::from(0u128), U256::from(1u128), U256::from(2u128)]))]
#[case(packUnpackPositionalNestedVectorsCall::new((0u32, 0u64)), (vec![vec![0u32, 1u32, 2u32], vec![3u32, 4u32, 5u32]], vec![vec![0u64, 1u64, 2u64], vec![3u64, 4u64, 5u64]]))]
fn test_pack_unpack_enums_with_vectors<T: SolCall, V: SolValue>(
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

#[rstest]
#[case(packUnpackAlphaCall::new((0, 0u16, 0u32, 0u64)), (0, 0u16, 0u32, 0u64))]
#[case(packUnpackAlphaCall::new((1, 2u16, 3u32, 4u64)), (1, 2u16, 3u32, 4u64))]
#[case(packUnpackAlphaCall::new((255, u16::MAX, u32::MAX, u64::MAX)), (255, u16::MAX, u32::MAX, u64::MAX))]
#[case(packUnpackBetaCall::new((0u128, U256::from(0u128))), (0u128, U256::from(0u128)))]
#[case(packUnpackBetaCall::new((1u128, U256::from(2u128))), (1u128, U256::from(2u128)))]
#[case(packUnpackBetaCall::new((u128::MAX, U256::from(u128::MAX))), (u128::MAX, U256::from(u128::MAX)))]
#[case(packUnpackGammaCall::new((vec![0, 1, 2], vec![false, true, false], 33u128, U256::from(42))), (vec![0, 1, 2], vec![false, true, false], 33u128, U256::from(42)))]
#[case(packUnpackGammaCall::new((vec![42u32, 43u32, 44u32], vec![true, false, true], 123u128, U256::from(321))), (vec![42u32, 43u32, 44u32], vec![true, false, true], 123u128, U256::from(321)))]
#[case(getGammaVecSumCall::new((vec![42u32, 43u32, 44u32], vec![true, false, true], 123u128, U256::from(321))), (129u32,))]
fn test_pack_unpack_struct_enums<T: SolCall, V: SolValue>(
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
