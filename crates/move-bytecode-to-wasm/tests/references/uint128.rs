use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_uint_128",
    "tests/references/move_sources/uint_128.move"
);

sol!(
    #[allow(missing_docs)]
    function derefU128(uint128 x) external returns (uint128);
    function derefU128Ref(uint128 x) external returns (uint128);
    function callDerefU128Ref(uint128 x) external returns (uint128);
    function derefNestedU128(uint128 x) external returns (uint128);
    function derefMutArg(uint128 x) external returns (uint128);
    function writeMutRef(uint128 x) external returns (uint128);
    function miscellaneous0() external returns (uint128[]);
    function miscellaneous1() external returns (uint128[]);
    function freezeRef(uint128 x) external returns (uint128[]);
    function identityU128Ref(uint128 x) external returns (uint128);
);

#[rstest]
#[case(derefU128Call::new((250,)), 250)]
#[case(derefU128RefCall::new((u128::MAX,)), u128::MAX)]
#[case(callDerefU128RefCall::new((1,)), 1)]
#[case(derefNestedU128Call::new((7,)), 7)]
#[case(derefMutArgCall::new((1,)), 1)]
#[case(writeMutRefCall::new((2,)), 1)]
#[case(freezeRefCall::new((3,)), 3)]
#[case(identityU128RefCall::new((4,)), 4)]
fn test_uint_128_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u128,
) {
    let expected_result = <sol!((uint128,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(miscellaneous0Call::new(()), vec![1u128, 2u128, 3u128])]
#[case(miscellaneous1Call::new(()), vec![1u128, 2u128, 3u128])]
fn test_uint_128_mut_ref_misc<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<u128>,
) {
    let expected_result = <sol!(uint128[])>::abi_encode(&expected_result);
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
