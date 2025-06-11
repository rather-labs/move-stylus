use alloy_sol_types::SolValue;
use alloy_sol_types::abi::TokenSeq;
use alloy_sol_types::{SolCall, SolType, sol};
use anyhow::Result;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};
use rstest::{fixture, rstest};

mod common;

fn run_test(runtime: &RuntimeSandbox, call_data: Vec<u8>, expected_result: Vec<u8>) -> Result<()> {
    let (result, return_data) = runtime.call_entrypoint(call_data)?;
    anyhow::ensure!(
        result == 0,
        "Function returned non-zero exit code: {result}"
    );
    anyhow::ensure!(
        return_data == expected_result,
        "return data mismatch:\nreturned:{return_data:?}\nexpected:{expected_result:?}"
    );

    Ok(())
}

#[fixture]
#[once]
fn runtime() -> RuntimeSandbox {
    const MODULE_NAME: &str = "structs";
    const SOURCE_PATH: &str = "tests/structs/struct.move";

    let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);

    RuntimeSandbox::new(&mut translated_package)
}

sol!(
    #[allow(missing_docs)]
    function echoBool(bool a) external returns (bool);
    function echoU64(uint64 a) external returns (uint64);
);

#[rstest]
#[case(echoBoolCall::new((true,)), (true,))]
#[case(echoBoolCall::new((false,)), (false,))]
#[case(echoU64Call::new((u64::MAX,)), (u64::MAX,))]
fn test_struct_field_reference<T: SolCall, V: SolValue>(
    #[by_ref] runtime: &RuntimeSandbox,
    #[case] call_data: T,
    #[case] expected_result: V,
) where
    for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
{
    run_test(
        runtime,
        call_data.abi_encode(),
        expected_result.abi_encode_params(),
    )
    .unwrap();
}
