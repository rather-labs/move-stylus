use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_vec_64",
    "tests/references/move_sources/vec_64.move"
);

sol!(
    #[allow(missing_docs)]
    function deref(uint64[] x) external returns (uint64[]);
    function derefArg(uint64[] x) external returns (uint64[]);
    function callDerefArg(uint64[] x) external returns (uint64[]);
    function vecFromElement(uint64 index) external returns (uint64[]);
    function getElementVector(uint64 index) external returns (uint64[]);
    function derefMutArg(uint64[] x) external returns (uint64[]);
    function writeMutRef(uint64[] x) external returns (uint64[]);
    function miscellaneous0() external returns (uint64[]);
    function miscellaneous1() external returns (uint64[]);
    function miscellaneous2() external returns (uint64[]);
    function miscellaneous3(uint64[] x) external returns (uint64[]);
    function miscellaneous4() external returns (uint64[]);
    function freezeRef(uint64[] x) external returns (uint64[]);
    function identityVecRef(uint64[] x) external returns (uint64[]);
);

#[rstest]
#[case(derefCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
#[case(derefArgCall::new((vec![4, 5, 6],)), vec![4, 5, 6])]
#[case(callDerefArgCall::new((vec![7, 8, 9],)), vec![7, 8, 9])]
#[case(vecFromElementCall::new((0,)), vec![10])]
#[case(getElementVectorCall::new((0,)), vec![10, 20])]
#[case(derefMutArgCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
#[case(writeMutRefCall::new((vec![4, 5, 6],)), vec![1, 2, 3])]
#[case(freezeRefCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
#[case(miscellaneous0Call::new(()), vec![4, 5, 4])]
#[case(miscellaneous1Call::new(()), vec![20, 40])]
#[case(miscellaneous2Call::new(()), vec![1, 4, 7])]
#[case(miscellaneous3Call::new((vec![1, 2, 3],)), vec![99, 1, 3])]
#[case(miscellaneous4Call::new(()), vec![1, 12, 111, 12, 11, 112])]
#[case(identityVecRefCall::new((vec![1, 2, 3],)), vec![1, 2, 3])]
fn test_vec_64_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<u64>,
) {
    let expected_result = <sol!(uint64[])>::abi_encode(&expected_result);
    println!("expected_result: {expected_result:?}");
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
