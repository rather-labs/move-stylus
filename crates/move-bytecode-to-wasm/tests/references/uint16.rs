use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!(
    "references_uint_16",
    "tests/references/move_sources/uint_16.move"
);

sol!(
    #[allow(missing_docs)]
    function derefU16(uint16 x) external returns (uint16);
    function derefU16Ref(uint16 x) external returns (uint16);
    function callDerefU16Ref(uint16 x) external returns (uint16);
    function derefNestedU16(uint16 x) external returns (uint16);
    function derefMutArg(uint16 x) external returns (uint16);
    function writeMutRef(uint16 x) external returns (uint16);
    function miscellaneous0() external returns (uint16[]);
    function miscellaneous1() external returns (uint16[]);
    function freezeRef(uint16 x) external returns (uint16);
    function identityU16Ref(uint16 x) external returns (uint16);
);

#[rstest]
#[case(derefU16Call::new((250,)), 250)]
#[case(derefU16RefCall::new((u16::MAX,)), u16::MAX)]
#[case(callDerefU16RefCall::new((1,)), 1)]
#[case(derefNestedU16Call::new((7,)), 7)]
#[case(derefMutArgCall::new((1,)), 1)]
#[case(writeMutRefCall::new((2,)), 1)]
#[case(freezeRefCall::new((3,)), 3)]
#[case(identityU16RefCall::new((4,)), 4)]
fn test_uint_16_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: u16,
) {
    let expected_result = <sol!((uint16,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(miscellaneous0Call::new(()), vec![1u16, 2u16, 3u16])]
#[case(miscellaneous1Call::new(()), vec![1u16, 2u16, 3u16])]
fn test_uint_16_mut_ref_misc<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<u16>,
) {
    let expected_result = <sol!(uint16[])>::abi_encode(&expected_result);
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
