use crate::common::run_test;
use crate::declare_fixture;
use alloy_sol_types::{SolCall, SolType, SolValue, abi::TokenSeq, sol};
use move_test_runner::wasm_runner::RuntimeSandbox;
use rstest::{fixture, rstest};

declare_fixture!("string_utf8", "tests/stdlib/move_sources/string_utf8.move");

sol!(
    #[allow(missing_docs)]
    function packUtf8() external returns (string);
);

#[rstest]
#[case(packUtf8Call::new(()), "hello world")]
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
