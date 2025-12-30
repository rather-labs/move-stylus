use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_uint_64",
    "tests/references/move_sources/uint_64.move"
);

sol!(
    #[allow(missing_docs)]
    function derefU64(uint64 x) external returns (uint64);
    function derefU64Ref(uint64 x) external returns (uint64);
    function callDerefU64Ref(uint64 x) external returns (uint64);
    function derefNestedU64(uint64 x) external returns (uint64);
    function derefMutArg(uint64 x) external returns (uint64);
    function writeMutRef(uint64 x) external returns (uint64);
    function miscellaneous0() external returns (uint64[]);
    function miscellaneous1() external returns (uint64[]);
    function freezeRef(uint64 x) external returns (uint64[]);
    function identityU64Ref(uint64 x) external returns (uint64);
);

#[rstest]
#[case(derefU64Call::new((250,)), 250)]
#[case(derefU64RefCall::new((u64::MAX,)), u64::MAX)]
#[case(callDerefU64RefCall::new((1,)), 1)]
#[case(derefNestedU64Call::new((7,)), 7)]
#[case(derefMutArgCall::new((1,)), 1)]
#[case(writeMutRefCall::new((2,)), 1)]
#[case(freezeRefCall::new((3,)), 3)]
#[case(identityU64RefCall::new((4,)), 4)]
fn test_uint_64_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u64,
) {
    let expected_result = <sol!((uint64,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(miscellaneous0Call::new(()), vec![1u64, 2u64, 3u64])]
#[case(miscellaneous1Call::new(()), vec![1u64, 2u64, 3u64])]
fn test_uint_64_mut_ref_misc<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<u64>,
) {
    let expected_result = <sol!(uint64[])>::abi_encode(&expected_result);
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
