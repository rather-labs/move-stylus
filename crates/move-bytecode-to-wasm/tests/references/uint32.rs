use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_uint_32",
    "tests/references/move_sources/uint_32.move"
);

sol!(
    #[allow(missing_docs)]
    function derefU32(uint32 x) external returns (uint32);
    function derefU32Ref(uint32 x) external returns (uint32);
    function callDerefU32Ref(uint32 x) external returns (uint32);
    function derefNestedU32(uint32 x) external returns (uint32);
    function derefMutArg(uint32 x) external returns (uint32);
    function writeMutRef(uint32 x) external returns (uint32);
    function miscellaneous0() external returns (uint32[]);
    function miscellaneous1() external returns (uint32[]);
    function freezeRef(uint32 x) external returns (uint32[]);
    function identityU32Ref(uint32 x) external returns (uint32);
);

#[rstest]
#[case(derefU32Call::new((250,)), 250)]
#[case(derefU32RefCall::new((u32::MAX,)), u32::MAX)]
#[case(callDerefU32RefCall::new((1,)), 1)]
#[case(derefNestedU32Call::new((7,)), 7)]
#[case(derefMutArgCall::new((1,)), 1)]
#[case(writeMutRefCall::new((2,)), 1)]
#[case(freezeRefCall::new((3,)), 3)]
#[case(identityU32RefCall::new((4,)), 4)]
fn test_uint_32_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u32,
) {
    let expected_result = <sol!((uint32,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(miscellaneous0Call::new(()), vec![1u32, 2u32, 3u32])]
#[case(miscellaneous1Call::new(()), vec![1u32, 2u32, 3u32])]
fn test_uint_32_mut_ref_misc<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<u32>,
) {
    let expected_result = <sol!(uint32[])>::abi_encode(&expected_result);
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
