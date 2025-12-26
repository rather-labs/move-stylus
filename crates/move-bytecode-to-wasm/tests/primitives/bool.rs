use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("bool_type", "tests/primitives/move_sources/bool.move");

sol!(
    #[allow(missing_docs)]
    function getConstant() external returns (bool);
    function getLocal(bool _z) external returns (bool);
    function getCopiedLocal() external returns (bool, bool);
    function echo(bool x) external returns (bool);
    function echo2(bool x, bool y) external returns (bool);
    function notTrue() external returns (bool);
    function not(bool x) external returns (bool);
);

#[rstest]
#[case(getConstantCall::new(()), (true,))]
#[case(getLocalCall::new((true,)), (false,))]
#[case(getCopiedLocalCall::new(()), (true, false))]
#[case(echoCall::new((true,)), (true,))]
#[case(echoCall::new((false,)), (false,))]
#[case(echo2Call::new((true, false)), (false,))]
#[case(notTrueCall::new(()), (false,))]
#[case(notCall::new((false,)), (true,))]
#[case(notCall::new((true,)), (false,))]
fn test_bool<T: SolCall, V: SolValue>(
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
