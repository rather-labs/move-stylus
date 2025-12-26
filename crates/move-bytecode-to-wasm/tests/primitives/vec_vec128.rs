use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "vec_vec_128",
    "tests/primitives/move_sources/vec_vec_128.move"
);

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint128[][]);
    function getConstantLocal() external returns (uint128[][]);
    function getLiteral() external returns (uint128[][]);
    function getCopiedLocal() external returns (uint128[][]);
    function echo(uint128[][] x) external returns (uint128[][]);
    function vecLen(uint128[][] x) external returns (uint64);
    function vecPopBack(uint128[][] x) external returns (uint128[][]);
    function vecSwap(uint128[][] x, uint64 id1, uint64 id2) external returns (uint128[][]);
    function vecPushBack(uint128[][] x, uint128[] y) external returns (uint128[][]);
    function vecPushBackToElement(uint128[][] x, uint128 y) external returns (uint128[][]);
    function vecPushAndPopBack(uint128[][] x, uint128[] y) external returns (uint128[][]);
    function misc0(uint128[][] x, uint128 y) external returns (uint128[][]);
    function vecUnpack(uint128[][] x) external returns (uint128[][]);
);

#[rstest]
#[case(getConstantCall::new(()), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
#[case(getConstantLocalCall::new(()), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
#[case(getLiteralCall::new(()), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
#[case(getCopiedLocalCall::new(()), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
#[case(echoCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]],)), vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]])]
#[case(vecLenCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]],)), (3u64,))]
#[case(vecPopBackCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]],)), vec![vec![1u128, 2u128, 3u128],])]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(vecPopBackCall::new((vec![],)), ((),))]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(vecSwapCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]], 0u64, 3u64)), ((),))]
#[case(vecSwapCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]], 0u64, 1u64)), vec![vec![4u128, 5u128, 6u128], vec![1u128, 2u128, 3u128], vec![7u128, 8u128, 9u128]])]
#[case(vecSwapCall::new((vec![vec![1u128, 2u128, 3u128], vec![4u128, 5u128, 6u128], vec![7u128, 8u128, 9u128]], 0u64, 2u64)), vec![vec![7u128, 8u128, 9u128], vec![4u128, 5u128, 6u128], vec![1u128, 2u128, 3u128]])]
#[case(vecPushBackCall::new((vec![vec![1u128, 2u128], vec![3u128, 4u128]], vec![5u128, 6u128])), vec![vec![1u128, 2u128], vec![3u128, 4u128], vec![5u128, 6u128], vec![5u128, 6u128]])]
#[case(vecPushAndPopBackCall::new((vec![vec![1u128, 2u128], vec![3u128, 4u128]], vec![5u128, 6u128])), vec![vec![1u128, 2u128], vec![3u128, 4u128]])]
#[case(misc0Call::new((vec![vec![1u128, 2u128], vec![3u128, 4u128]], 99u128)), vec![vec![1u128, 2u128, 99u128], vec![4u128, 99u128]])]
#[case(vecUnpackCall::new((vec![vec![1u128], vec![5u128], vec![9u128]],)), vec![vec![3], vec![1], vec![4], vec![1], vec![5], vec![9]])]
fn test_vec_vec_128<T: SolCall, V: SolValue>(
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
