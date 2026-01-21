use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("vec_32", "tests/primitives/move_sources/vec_32.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint32[]);
    function getConstantLocal() external returns (uint32[]);
    function getLiteral() external returns (uint32[]);
    function getCopiedLocal() external returns (uint32[]);
    function echo(uint32[] x) external returns (uint32[]);
    function vecFromInt(uint32 x, uint32 y) external returns (uint32[]);
    function vecFromVec(uint32[] x, uint32[] y) external returns (uint32[][]);
    function vecFromVecAndInt(uint32[] x, uint32 y) external returns (uint32[][]);
    function vecLen(uint32[] x) external returns (uint64);
    function vecPopBack(uint32[] x) external returns (uint32[]);
    function vecSwap(uint32[] x, uint64 id1, uint64 id2) external returns (uint32[]);
    function vecPushBack(uint32[] x, uint32 y) external returns (uint32[]);
    function vecPushAndPopBack(uint32[] x, uint32 y) external returns (uint32[]);
    function vecUnpack(uint32[] x) external returns (uint32[]);
    function cumulativeSum(uint32[] x) external returns (uint32);
    function vecAppend(uint32[] x, uint32[] y) external returns (uint32[]);
    function vecAppend2(uint32[] x, uint32[] y) external returns (uint32[]);
    function testMutateMutRefVector(uint32[] x) external returns (uint32[]);
    function testMutateMutRefVector2(uint32[] x) external returns (uint32[]);
    function testContains(uint32[] v, uint32 e) external returns (bool);
    function testRemove(uint32[] v, uint64 index) external returns (uint32[]);
);

#[rstest]
#[case(getConstantCall::new(()), vec![1, 2, 3])]
#[case(getConstantLocalCall::new(()), vec![1, 2, 3])]
#[case(getLiteralCall::new(()), vec![1, 2, 3])]
#[case(getCopiedLocalCall::new(()), vec![1, 2, 3])]
#[case(echoCall::new((vec![1u32, 2u32, 3u32],)), vec![1, 2, 3])]
#[case(vecFromIntCall::new((1u32, 2u32)), vec![1, 2, 1])]
#[case(vecFromVecCall::new((vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32])), vec![vec![1, 2, 3], vec![4, 5, 6]])]
#[case(vecFromVecAndIntCall::new((vec![1u32, 2u32, 3u32], 4u32)), vec![vec![1, 2, 3], vec![4, 4]])]
#[case(vecLenCall::new((vec![1u32, 2u32, 3u32],)), (3u64,))]
#[case(vecPopBackCall::new((vec![1u32, 2u32, 3u32],)), vec![1])]
#[case(vecSwapCall::new((vec![1u32, 2u32, 3u32], 0u64, 1u64)), vec![2, 1, 3])]
#[case(vecSwapCall::new((vec![1u32, 2u32, 3u32], 0u64, 2u64)), vec![3, 2, 1])]
#[case(vecPushBackCall::new((vec![1u32, 2u32, 3u32], 4u32)), vec![1, 2, 3, 4])]
#[case(vecPushAndPopBackCall::new((vec![1u32, 2u32, 3u32], 4u32)), vec![1, 2, 3])]
#[case(vecUnpackCall::new((vec![1u32, 5u32, 9u32],)), vec![3, 1, 4, 1, 5, 9])]
#[case(cumulativeSumCall::new((vec![1u32, 2u32, 3u32],)), (6u32,))]
#[case(vecAppendCall::new((vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32])), vec![1, 2, 3, 4, 5, 6])]
#[case(vecAppend2Call::new((vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32, 7u32])), vec![1, 2, 3, 4, 5, 6, 7])]
#[case(testMutateMutRefVectorCall::new((vec![1u32],)), vec![1, 42, 43, 44])]
#[case(testMutateMutRefVector2Call::new((vec![1u32],)), vec![1, 42, 43, 44])]
#[case(testContainsCall::new((vec![1u32, 2u32, 3u32], 2u32)), (true,))]
#[case(testContainsCall::new((vec![1u32, 2u32, 3u32], 4u32)), (false,))]
#[case(testRemoveCall::new((vec![1u32, 2u32, 3u32], 1u64)), vec![1, 3])]
fn test_vec_32<T: SolCall, V: SolValue>(
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
#[case(vecSwapCall::new((vec![1u32, 2u32, 3u32], 0u64, 3u64)))]
#[case(vecPopBackCall::new((vec![],)))]
fn test_vec_32_runtime_error<T: SolCall>(#[by_ref] runtime: &RuntimeSandbox, #[case] call_data: T) {
    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    assert_eq!(result, 1);

    let expected_data = RuntimeError::OutOfBounds.encode_abi();
    assert_eq!(return_data, expected_data);
}
