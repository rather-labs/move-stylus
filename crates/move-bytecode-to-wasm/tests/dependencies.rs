use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use anyhow::Result;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_complete_package};
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
    let mut translated_packages = translate_test_complete_package("tests/dependencies");

    let translated_package = translated_packages.get_mut("main").unwrap();
    RuntimeSandbox::new(translated_package)
}

sol!(
    struct AnotherTest {
        uint8 at_field;
    }

    struct Test {
        uint8 t_field_1;
        AnotherTest t_field_2;
    }

    #[allow(missing_docs)]
    function echoTestStruct(Test a) external returns (uint8, uint8);
    function echoAnotherTestStruct(AnotherTest a) external returns (uint8);
);

/// This tests that the internal modules of the packages can see each other and depend on each
/// other. It should compile all the three .move files inside the dependencies folder without
/// failing.
/// The dependency tree is as follows:
/// - another_mod.move: No dependencies
/// - other_mod.move: depends on
///     - another_mod.move
/// - main.move: depends on
///     - another_mod.move
///     - other_mod.move
#[rstest]
#[case(echoTestStructCall::new((
    Test {
        t_field_1: 42,
        t_field_2: AnotherTest { at_field: 84 }
    },
    )),
    (42,84)
)]
#[case(echoAnotherTestStructCall::new((
    AnotherTest { at_field: 100 },
    )),
    (100,)
)]
fn test_dependencies<T: SolCall, V: SolValue>(
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
