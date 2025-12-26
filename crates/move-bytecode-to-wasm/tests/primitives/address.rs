use crate::common::run_test;
use crate::declare_fixture;
use alloy_primitives::address;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("address_type", "tests/primitives/move_sources/address.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (address);
    function getLocal(address _z) external returns (address);
    function getCopiedLocal() external returns (address, address);
    function echo(address x) external returns (address);
    function echo2(address x, address y) external returns (address);
);

#[rstest]
#[case(getConstantCall::new(()), (address!("0x0000000000000000000000000000000000000001"),))]
#[case(
        getLocalCall::new((address!("0x0000000000000000000000000000000000000022"),)),
        (address!("0x0000000000000000000000000000000000000011"),)
    )]
#[case(
        getCopiedLocalCall::new(()),
        (
            address!("0x0000000000000000000000000000000000000001"),
            address!("0x0000000000000000000000000000000000000022")
        )
    )]
#[case(
        echoCall::new((address!("0x0000000000000000000000000000000000000033"),)),
        (address!("0x0000000000000000000000000000000000000033"),)
    )]
#[case(
        echoCall::new((address!("0x0000000000000000000000000000000000000044"),)),
        (address!("0x0000000000000000000000000000000000000044"),)
    )]
#[case(
        echo2Call::new((
            address!("0x0000000000000000000000000000000000000055"),
            address!("0x0000000000000000000000000000000000000066"),
        )),
        ( address!("0x0000000000000000000000000000000000000066"),)
    )]
fn test_address<T: SolCall, V: SolValue>(
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
