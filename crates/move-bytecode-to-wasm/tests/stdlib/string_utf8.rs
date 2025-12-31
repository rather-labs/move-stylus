use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("string_utf8", "tests/stdlib/move_sources/string_utf8.move");

sol!(
    #[allow(missing_docs)]
    function packUtf8(uint8[] string_bytes) external returns (string);
    function packUtf82() external returns (string, string);
    function packUtf83() external returns (string, uint16, string);
    function packUtf84() external returns (string, uint16[], string);
    function unpackUtf8(string value) external returns (bool); function unpackUtf82(string value, string value2) external returns (bool);
    function unpackUtf83(string value, uint16 n, string value2) external returns (bool);
    function unpackUtf84(string value, uint16[] n, string value2) external returns (bool);
    function packUnpackUtf8(string value) external returns (string); function packUnpackUtf82(string value, string value2) external returns (string, string);
);

#[rstest]
// 1-byte UTF-8
#[case(packUtf8Call::new((b"hello world".to_vec(),)), "hello world")]
// 2-byte UTF-8
#[case(packUtf8Call::new(("Привет мир".as_bytes().to_vec(),)), "Привет мир")]
// 3-byte UTF-8
#[case(packUtf8Call::new(("こんにちは 世界".as_bytes().to_vec(),)), "こんにちは 世界")]
// 4-byte UTF-8
#[case(packUtf8Call::new(("🐱😊😎😿😻".as_bytes().to_vec(),)), "🐱😊😎😿😻")]
// Mixed UTF-8
#[case(packUtf8Call::new(("Hello, 世界! 👋".as_bytes().to_vec(),)), "Hello, 世界! 👋")]
#[case(unpackUtf8Call::new(("dlrow olleh".to_owned(),)), true)]
#[case(unpackUtf82Call::new((
        "hello world".to_owned(),
        "test string".to_owned(),
    )), true)]
#[case(unpackUtf83Call::new((
        "hello world".to_owned(),
        42,
        "test string".to_owned(),
    )), true)]
#[case(unpackUtf84Call::new((
        "hello world".to_owned(),
        vec![3,1,4,1,5],
        "test string".to_owned(),
    )), true)]
#[case(packUnpackUtf8Call::new(("test string".to_owned(),)), "test string")]
fn test_utf8<T: SolCall, V: SolValue>(
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
#[case(packUtf82Call::new(()), ("Привет мир", "こんにちは 世界"))]
#[case(packUtf83Call::new(()), ("hello world", 42, "test string"))]
#[case(packUtf84Call::new(()), ("hello world", vec![3,1,4,1,5], "test string"))]
#[case(packUnpackUtf82Call::new((
        "test string".to_owned(),
        "hello world".to_owned()
    )), (
        "test string",
        "hello world",
    ))]
fn test_utf8_multiple<T: SolCall, V: SolValue>(
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
