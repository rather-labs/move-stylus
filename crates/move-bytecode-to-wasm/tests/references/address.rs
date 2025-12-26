use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::Address;
use alloy_primitives::address;
use alloy_sol_types::{SolCall, SolType, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("ref_address", "tests/references/move_sources/address.move");

sol!(
    #[allow(missing_docs)]
    function derefAddress(address x) external returns (address);
    function derefAddressRef(address x) external returns (address);
    function callDerefAddressRef(address x) external returns (address);
    function derefNestedAddress(address x) external returns (address);
    function derefMutArg(address x) external returns (address);
    function writeMutRef(address x) external returns (address);
    function miscellaneous0() external returns (address[]);
    function miscellaneous1() external returns (address[]);
    function freezeRef(address x) external returns (address);
    function identityAddressRef(address x) external returns (address);
);

#[rstest]
#[case(derefAddressCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x1234567890abcdef1234567890abcdef12345678"))]
#[case(callDerefAddressRefCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x1234567890abcdef1234567890abcdef12345678"))]
#[case(derefNestedAddressCall::new((address!("0x7890abcdef1234567890abcdef1234567890abcd"),)), address!("0x7890abcdef1234567890abcdef1234567890abcd"))]
#[case(derefMutArgCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x1234567890abcdef1234567890abcdef12345678"))]
#[case(writeMutRefCall::new((address!("0x1234567890abcdef1234567890abcdef12345678"),)), address!("0x0000000000000000000000000000000000000001"))]
#[case(freezeRefCall::new((address!("0x0000000000000000000000000000000000000003"),)), address!("0x0000000000000000000000000000000000000003"))]
#[case(identityAddressRefCall::new((address!("0x0000000000000000000000000000000000000004"),)), address!("0x0000000000000000000000000000000000000004"))]
fn test_address_ref<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Address,
) {
    let expected_result = <sol!((address,))>::abi_encode(&(expected_result,));
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}

#[rstest]
#[case(miscellaneous0Call::new(()), vec![address!("0x0000000000000000000000000000000000000001"), address!("0x0000000000000000000000000000000000000002"), address!("0x0000000000000000000000000000000000000002")])]
#[case(miscellaneous1Call::new(()), vec![address!("0x0000000000000000000000000000000000000001"), address!("0x0000000000000000000000000000000000000003"), address!("0x0000000000000000000000000000000000000002")])]
fn test_address_ref_misc<T: SolCall>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: Vec<Address>,
) {
    let expected_result = <sol!(address[])>::abi_encode(&expected_result);
    run_test(runtime, call_data.abi_encode(), expected_result).unwrap();
}
