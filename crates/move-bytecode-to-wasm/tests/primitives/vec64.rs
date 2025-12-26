use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("vec_64", "tests/primitives/move_sources/vec_64.move");

sol!(
      #[allow(missing_docs)]
      function getConstant() external returns (uint64[]);
      function getConstantLocal() external returns (uint64[]);
      function getLiteral() external returns (uint64[]);
      function getCopiedLocal() external returns (uint64[]);
      function echo(uint64[] x) external returns (uint64[]);
      function vecFromInt(uint64 x, uint64 y) external returns (uint64[]);
      function vecFromVec(uint64[] x, uint64[] y) external returns (uint64[][]);
      function vecFromVecAndInt(uint64[] x, uint64 y) external returns (uint64[][]);
      function vecLen(uint64[] x) external returns (uint64);
      function vecPopBack(uint64[] x) external returns (uint64[]);
      function vecSwap(uint64[] x, uint64 id1, uint64 id2) external returns (uint64[]);
      function vecPushBack(uint64[] x, uint64 y) external returns (uint64[]);
      function vecPushAndPopBack(uint64[] x, uint64 y) external returns (uint64[]);
      function vecUnpack(uint64[] x) external returns (uint64[]);
);

#[rstest]
#[case(getConstantCall::new(()), vec![1u64, 2u64, 3u64])]
#[case(getConstantLocalCall::new(()), vec![1u64, 2u64, 3u64])]
#[case(getLiteralCall::new(()), vec![1u64, 2u64, 3u64])]
#[case(getCopiedLocalCall::new(()), vec![1u64, 2u64, 3u64])]
#[case(echoCall::new((vec![1u64, 2u64, 3u64],)), vec![1u64, 2u64, 3u64])]
#[case(vecFromIntCall::new((1u64, 2u64)), vec![1u64, 2u64, 1u64 ])]
#[case(vecFromVecCall::new((vec![1u64, 2u64, 3u64], vec![4u64, 5u64, 6u64])), vec![vec![1u64, 2u64, 3u64], vec![4u64, 5u64, 6u64]])]
#[case(vecFromVecAndIntCall::new((vec![1u64, 2u64, 3u64], 4u64)), vec![vec![1, 2, 3], vec![4, 4]])]
#[case(vecLenCall::new((vec![1u64, 2u64, 3u64],)), (3u64,))]
#[case(vecPopBackCall::new((vec![1u64, 2u64, 3u64],)), vec![1])]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(vecPopBackCall::new((vec![],)), ((),))]
#[should_panic(expected = r#"wasm trap: wasm `unreachable` instruction executed"#)]
#[case(vecSwapCall::new((vec![1u64, 2u64, 3u64], 0u64, 3u64)), ((),))]
#[case(vecSwapCall::new((vec![1u64, 2u64, 3u64], 0u64, 1u64)), vec![2u64, 1u64, 3u64])]
#[case(vecSwapCall::new((vec![1u64, 2u64, 3u64], 0u64, 2u64)), vec![3u64, 2u64, 1u64])]
#[case(vecPushBackCall::new((vec![1u64, 2u64, 3u64], 4u64)), vec![1u64, 2u64, 3u64, 4u64, 4u64])]
#[case(vecPushAndPopBackCall::new((vec![1u64, 2u64, 3u64], 4u64)), vec![1u64, 2u64, 3u64])]
#[case(vecUnpackCall::new((vec![1u64, 5u64, 9u64],)), vec![3, 1, 4, 1, 5, 9])]
fn test_vec_64<T: SolCall, V: SolValue>(
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
