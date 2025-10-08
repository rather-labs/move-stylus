mod common;

use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use anyhow::Result;
use common::{runtime_sandbox::RuntimeSandbox, translate_test_package};
use rstest::{fixture, rstest};

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

/// This test is here to check if code that use the standard library gets compiled to Move
/// Bytecode.
#[test]
fn test_use_stdlib() {
    const MODULE_NAME: &str = "use_stdlib";
    const SOURCE_PATH: &str = "tests/stdlib/use_stdlib.move";

    translate_test_package(SOURCE_PATH, MODULE_NAME);
}

mod string {
    use super::*;

    const MODULE_NAME: &str = "string";
    const SOURCE_PATH: &str = "tests/stdlib/string.move";

    #[fixture]
    #[once]
    fn runtime() -> RuntimeSandbox {
        let mut translated_package = translate_test_package(SOURCE_PATH, MODULE_NAME);
        RuntimeSandbox::new(&mut translated_package)
    }

    sol!(
        #[allow(missing_docs)]
        function packAscii() external returns (string);
        function packAscii2() external returns (string, string);
        function packAscii3() external returns (string, uint16, string);
        function packAscii4() external returns (string, uint16[], string);
        function unpackAscii(string value) external returns (bool);
        function unpackAscii2(string value, string value2) external returns (bool);
        function packUnpackAscii(string value) external returns (string);
        function packUnpackAscii2(string value, string value2) external returns (string, string);
    );

    #[rstest]
    #[case(packAsciiCall::new(()), "hello world")]
    #[case(unpackAsciiCall::new(("dlrow olleh".to_owned(),)), true)]
    #[case(unpackAscii2Call::new((
        "hello world".to_owned(),
        "test string".to_owned(),
    )), true)]
    #[case(packUnpackAsciiCall::new(("test string".to_owned(),)), "test string")]
    fn test_ascii<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode(),
        )
        .unwrap();
    }

    #[rstest]
    #[case(packAscii2Call::new(()), ("hello world", "test string"))]
    #[case(packAscii3Call::new(()), ("hello world", 42, "test string"))]
    #[case(packAscii4Call::new(()), ("hello world", vec![3,1,4,1,5], "test string"))]
    #[case(packUnpackAscii2Call::new((
        "test string".to_owned(),
        "hello world".to_owned()
    )), (
        "test string",
        "hello world",
    ))]
    fn test_ascii_multiple<T: SolCall, V: SolValue>(
        #[by_ref] runtime: &RuntimeSandbox,
        #[case] call_data: T,
        #[case] expected_result: V,
    ) where
        for<'a> <V::SolType as SolType>::Token<'a>: TokenSeq<'a>,
    {
        run_test(
            runtime,
            call_data.abi_encode(),
            expected_result.abi_encode_sequence(),
        )
        .unwrap();
    }
}
