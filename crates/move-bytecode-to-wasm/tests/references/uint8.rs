use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_uint_8",
    "tests/references/move_sources/uint_8.move"
);

sol!(
    #[allow(missing_docs)]
    function derefU8(uint8 x) external returns (uint8);
    function derefU8Ref(uint8 x) external returns (uint8);
    function callDerefU8Ref(uint8 x) external returns (uint8);
    function derefNestedU8(uint8 x) external returns (uint8);
    function derefMutArg(uint8 x) external returns (uint8);
    function writeMutRef(uint8 x) external returns (uint8);
    function miscellaneous0() external returns (uint8[]);
    function miscellaneous1() external returns (uint8[]);
    function freezeRef(uint8 x) external returns (uint8);
    function identityU8Ref(uint8 x) external returns (uint8);
);

#[rstest]
#[case(derefU8Call::new((250,)), 250)]
#[case(derefU8RefCall::new((u8::MAX,)), u8::MAX)]
#[case(callDerefU8RefCall::new((1,)), 1)]
#[case(derefNestedU8Call::new((7,)), 7)]
#[case(derefMutArgCall::new((1,)), 1)]
#[case(writeMutRefCall::new((2,)), 1)]
#[case(freezeRefCall::new((3,)), 3)]
#[case(identityU8RefCall::new((4,)), 4)]
fn test_uint_8_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u8,
) {
    let expected_result = <sol!((uint8,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(miscellaneous0Call::new(()), vec![1u8, 2u8, 3u8])]
#[case(miscellaneous1Call::new(()), vec![1u8, 2u8, 3u8])]
fn test_uint_8_mut_ref_misc<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<u8>,
) {
    let expected_result = <sol!(uint8[])>::abi_encode(&expected_result);
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
