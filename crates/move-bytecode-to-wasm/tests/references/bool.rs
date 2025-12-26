use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("references_bool", "tests/references/move_sources/bool.move");

sol!(
    #[allow(missing_docs)]
    function derefBool(bool x) external returns (bool);
    function derefBoolRef(bool x) external returns (bool);
    function callDerefBoolRef(bool x) external returns (bool);
    function derefNestedBool(bool x) external returns (bool);
    function derefMutArg(bool x) external returns (bool);
    function writeMutRef(bool x) external returns (bool);
    function miscellaneous0() external returns (bool[]);
    function miscellaneous1() external returns (bool[]);
    function identityBoolRef(bool x) external returns (bool);
);

#[rstest]
#[case(derefBoolCall::new((true,)), true)]
#[case(derefBoolRefCall::new((false,)), false)]
#[case(callDerefBoolRefCall::new((true,)), true)]
#[case(derefNestedBoolCall::new((false,)), false)]
#[case(derefMutArgCall::new((true,)), true)]
#[case(writeMutRefCall::new((false,)), true)]
#[case(identityBoolRefCall::new((true,)), true)]
fn test_bool_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: bool,
) {
    let expected_result = <sol!((bool,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(miscellaneous0Call::new(()), vec![false, true, false])]
#[case(miscellaneous1Call::new(()), vec![true, true, false])]
fn test_bool_ref_misc<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<bool>,
) {
    let expected_result = <sol!(bool[])>::abi_encode(&expected_result);
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
