use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_uint_256",
    "tests/references/move_sources/uint_256.move"
);

sol!(
    #[allow(missing_docs)]
    function derefU256(uint256 x) external returns (uint256);
    function derefU256Ref(uint256 x) external returns (uint256);
    function callDerefU256Ref(uint256 x) external returns (uint256);
    function derefNestedU256(uint256 x) external returns (uint256);
    function derefMutArg(uint256 x) external returns (uint256);
    function writeMutRef(uint256 x) external returns (uint256);
    function miscellaneous0() external returns (uint256[]);
    function miscellaneous1() external returns (uint256[]);
    function freezeRef(uint256 x) external returns (uint256[]);
    function identityU256Ref(uint256 x) external returns (uint256);
);

#[rstest]
#[case(derefU256Call::new((U256::from(250),)), U256::from(250))]
#[case(derefU256RefCall::new((U256::from(1234567890),)), U256::from(1234567890))]
#[case(callDerefU256RefCall::new((U256::from(1),)), U256::from(1))]
#[case(derefNestedU256Call::new((U256::from(7),)), U256::from(7))]
#[case(derefMutArgCall::new((U256::from(1),)), U256::from(1))]
#[case(writeMutRefCall::new((U256::from(2),)), U256::from(1))]
#[case(freezeRefCall::new((U256::from(3),)), U256::from(3))]
#[case(identityU256RefCall::new((U256::from(4),)), U256::from(4))]
fn test_uint_256_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: U256,
) {
    let expected_result = <sol!((uint256,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(miscellaneous0Call::new(()), vec![U256::from(1), U256::from(2), U256::from(3)])]
#[case(miscellaneous1Call::new(()), vec![U256::from(1), U256::from(2), U256::from(3)])]
fn test_uint_256_ref_misc<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<U256>,
) {
    let expected_result = <sol!(uint256[])>::abi_encode(&expected_result);
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
