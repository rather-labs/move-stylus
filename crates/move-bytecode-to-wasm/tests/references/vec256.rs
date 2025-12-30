use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::U256;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_vec_256",
    "tests/references/move_sources/vec_256.move"
);

sol!(
    #[allow(missing_docs)]
    function deref(uint256[] x) external returns (uint256[]);
    function derefArg(uint256[] x) external returns (uint256[]);
    function callDerefArg(uint256[] x) external returns (uint256[]);
    function vecFromElement(uint64 index) external returns (uint256[]);
    function getElementVector(uint64 index) external returns (uint256[]);
    function derefMutArg(uint256[] x) external returns (uint256[]);
    function writeMutRef(uint256[] x) external returns (uint256[]);
    function miscellaneous0() external returns (uint256[]);
    function miscellaneous1() external returns (uint256[]);
    function miscellaneous2() external returns (uint256[]);
    function miscellaneous3(uint256[] x) external returns (uint256[]);
    function miscellaneous4() external returns (uint256[]);
    function freezeRef(uint256[] x) external returns (uint256[]);
    function identityVecRef(uint256[] x) external returns (uint256[]);
);

#[rstest]
#[case(derefCall::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
#[case(derefArgCall::new((vec![U256::from(4), U256::from(5), U256::from(6)],)), vec![U256::from(4), U256::from(5), U256::from(6)])]
#[case(callDerefArgCall::new((vec![U256::from(7), U256::from(8), U256::from(9)],)), vec![U256::from(7), U256::from(8), U256::from(9)])]
#[case(vecFromElementCall::new((0,)), vec![U256::from(10)])]
#[case(getElementVectorCall::new((0,)), vec![U256::from(10), U256::from(20)])]
#[case(derefMutArgCall::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
#[case(writeMutRefCall::new((vec![U256::from(4), U256::from(5), U256::from(6)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
#[case(freezeRefCall::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
#[case(miscellaneous0Call::new(()), vec![U256::from(4), U256::from(5), U256::from(4)])]
#[case(miscellaneous1Call::new(()), vec![U256::from(20), U256::from(40)])]
#[case(miscellaneous2Call::new(()), vec![U256::from(1), U256::from(4), U256::from(7)])]
#[case(miscellaneous3Call::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(99), U256::from(1), U256::from(3)])]
#[case(miscellaneous4Call::new(()), vec![U256::from(1), U256::from(12), U256::from(111), U256::from(12), U256::from(11), U256::from(112)])]
#[case(identityVecRefCall::new((vec![U256::from(1), U256::from(2), U256::from(3)],)), vec![U256::from(1), U256::from(2), U256::from(3)])]
fn test_vec_256_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<U256>,
) {
    let expected_result = <sol!(uint256[])>::abi_encode(&expected_result);
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(getElementVectorCall::new((2,)))]
#[case(getElementVectorCall::new((u64::MAX,)))]
fn test_vec_256_out_of_bounds<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
) {
    run_test(runtime, call_data.abi_encode(), vec![])
        .expect_err("should fail")
        .to_string()
        .contains("wasm trap: wasm `unreachable` instruction executed");
}
