use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_bytecode_to_wasm::error::RuntimeError;
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "vec_vec_32",
    "tests/primitives/move_sources/vec_vec_32.move"
);

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (uint32[][]);
    function getConstantLocal() external returns (uint32[][]);
    function getLiteral() external returns (uint32[][]);
    function getCopiedLocal() external returns (uint32[][]);
    function echo(uint32[][] x) external returns (uint32[][]);
    function vecLen(uint32[][] x) external returns (uint64);
    function vecPopBack(uint32[][] x) external returns (uint32[][]);
    function vecSwap(uint32[][] x, uint64 id1, uint64 id2) external returns (uint32[][]);
    function vecPushBack(uint32[][] x, uint32[] y) external returns (uint32[][]);
    function vecPushBackToElement(uint32[][] x, uint32 y) external returns (uint32[][]);
    function vecPushAndPopBack(uint32[][] x, uint32[] y) external returns (uint32[][]);
    function misc0(uint32[][] x, uint32 y) external returns (uint32[][]);
    function vecUnpack(uint32[][] x) external returns (uint32[][]);
);

#[rstest]
#[case(getConstantCall::new(()), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
#[case(getConstantLocalCall::new(()), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
#[case(getLiteralCall::new(()), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
#[case(getCopiedLocalCall::new(()), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
#[case(echoCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]],)), vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]])]
#[case(vecLenCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]],)), (3u64,))]
#[case(vecPopBackCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]],)), vec![vec![1u32, 2u32, 3u32],])]
#[case(vecSwapCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]], 0u64, 1u64)), vec![vec![4u32, 5u32, 6u32], vec![1u32, 2u32, 3u32], vec![7u32, 8u32, 9u32]])]
#[case(vecSwapCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]], 0u64, 2u64)), vec![vec![7u32, 8u32, 9u32], vec![4u32, 5u32, 6u32], vec![1u32, 2u32, 3u32]])]
#[case(vecPushBackCall::new((vec![vec![1u32, 2u32], vec![3u32, 4u32]], vec![5u32, 6u32])), vec![vec![1u32, 2u32], vec![3u32, 4u32], vec![5u32, 6u32], vec![5u32, 6u32]])]
#[case(vecPushAndPopBackCall::new((vec![vec![1u32, 2u32], vec![3u32, 4u32]], vec![5u32, 6u32])), vec![vec![1u32, 2u32], vec![3u32, 4u32]])]
#[case(misc0Call::new((vec![vec![1u32, 2u32], vec![3u32, 4u32]], 99u32)), vec![vec![1u32, 2u32, 99u32], vec![4u32, 99u32]])]
#[case(vecUnpackCall::new((vec![vec![1u32], vec![5u32], vec![9u32]],)), vec![vec![3], vec![1], vec![4], vec![1], vec![5], vec![9]])]
fn test_vec_vec_32<T: SolCall, V: SolValue>(
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
#[case(vecPopBackCall::new((vec![],)),)]
#[case(vecSwapCall::new((vec![vec![1u32, 2u32, 3u32], vec![4u32, 5u32, 6u32], vec![7u32, 8u32, 9u32]], 0u64, 3u64)),)]
fn test_vec_vec_32_runtime_error<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
    let (result, return_data) = runtime.call_entrypoint(call_data.abi_encode()).unwrap();
    assert_eq!(result, 1);

    let expected_data = RuntimeError::OutOfBounds.encode_abi();
    assert_eq!(return_data, expected_data);
}
