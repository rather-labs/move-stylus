use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("vec_128", "tests/primitives/move_sources/vec_128.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint128[]);
    function getConstantLocal() external returns (uint128[]);
    function getLiteral() external returns (uint128[]);
    function getCopiedLocal() external returns (uint128[]);
    function echo(uint128[] x) external returns (uint128[]);
    function vecFromInt(uint128 x, uint128 y) external returns (uint128[]);
    function vecFromVec(uint128[] x, uint128[] y) external returns (uint128[][]);
    function vecFromVecAndInt(uint128[] x, uint128 y) external returns (uint128[][]);
    function vecLen(uint128[] x) external returns (uint64);
    function vecPopBack(uint128[] x) external returns (uint128[]);
    function vecSwap(uint128[] x, uint64 id1, uint64 id2) external returns (uint128[]);
    function vecPushBack(uint128[] x, uint128 y) external returns (uint128[]);
    function vecPushAndPopBack(uint128[] x, uint128 y) external returns (uint128[]);
    function vecUnpack(uint128[] x) external returns (uint128[]);
);

#[rstest]
#[case(getConstantCall::new(()), vec![1u128, 2u128, 3u128])]
#[case(getConstantLocalCall::new(()), vec![1u128, 2u128, 3u128])]
#[case(getLiteralCall::new(()), vec![1u128, 2u128, 3u128])]
#[case(getCopiedLocalCall::new(()), vec![1u128, 2u128, 3u128])]
#[case(echoCall::new((vec![1u128, 2u128, 3u128],)), vec![1u128, 2u128, 3u128])]
#[case(vecFromIntCall::new((1u128, 2u128)), vec![1u128, 2u128, 1u128])]
#[case(vecFromVecCall::new((vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128])), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128]])]
#[case(vecFromVecAndIntCall::new((vec![1u128, 2u128, 3u128], 4u128)), vec![vec![1u128, 2u128, 3u128], vec![4u128, 4u128]])]
#[case(vecLenCall::new((vec![1u128, 2u128, 3u128],)), (3u64,))]
#[case(vecPopBackCall::new((vec![1u128, 2u128, 3u128],)), vec![1u128])]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(vecPopBackCall::new((vec![],)), ((),))]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(vecSwapCall::new((vec![1u128, 2u128, 3u128], 0u64, 3u64)), ((),))]
#[case(vecSwapCall::new((vec![1u128, 2u128, 3u128], 0u64, 1u64)), vec![2u128, 1u128, 3u128])]
#[case(vecSwapCall::new((vec![1u128, 2u128, 3u128], 0u64, 2u64)), vec![3u128, 2u128, 1u128])]
#[case(vecPushBackCall::new((vec![1u128, 2u128, 3u128], 4u128)), vec![1u128, 2u128, 3u128, 4u128, 4u128])]
#[case(vecPushAndPopBackCall::new((vec![1u128, 2u128, 3u128], 4u128)), vec![1u128, 2u128, 3u128])]
#[case(vecUnpackCall::new((vec![1u128, 5u128, 9u128],)), vec![3, 1, 4, 1, 5, 9])]
fn test_vec_128<T: SolCall, V: SolValue>(
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
