use crate::common::run_test;
use crate::common::translate_test_package;
use crate::declare_fixture;
use alloy_primitives::address;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("signer_type", "tests/primitives/move_sources/signer.move");

sol!(
    #[allow(missing_docs)]
    function echo() external returns (address);
    function echoIdentity() external returns (address);
    function echoWithInt(uint8 y) external returns (uint8, address);
);

#[rstest]
#[should_panic]
#[case(echoCall::new(()), (address!("0x0000000000000000000000000000000007030507"),))]
#[should_panic]
#[case(echoIdentityCall::new(()), (address!("0x0000000000000000000000000000000007030507"),))]
#[should_panic]
#[case(echoWithIntCall::new((42,)), (42, address!("0x0000000000000000000000000000000007030507")))]
fn test_signer<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) where
    for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode(),
    )
    .unwrap();
}

#[rstest]
#[should_panic]
#[case("tests/primitives/move_sources/signer_invalid_dup_signer.move")]
#[should_panic]
#[case("tests/primitives/move_sources/signer_invalid_nested_signer.move")]
fn test_signer_invalid(#[case] path: &'static str) {
    translate_test_package(path, "signer_type");
}
