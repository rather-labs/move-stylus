use crate::common::translate_test_package;
use rstest::rstest;

#[rstest]
#[should_panic(expected = "ReceiveFunctionBadVisibility")]
#[case(
    "receive_bad_visibility",
    "tests/receive/move_sources/receive_bad_visibility.move"
)]
#[should_panic(expected = "ReceiveFunctionHasReturns")]
#[case(
    "receive_bad_returns",
    "tests/receive/move_sources/receive_bad_returns.move"
)]
#[should_panic(expected = "ReceiveFunctionTooManyArguments")]
#[case(
    "receive_bad_args_1",
    "tests/receive/move_sources/receive_bad_args_1.move"
)]
#[should_panic(expected = "ReceiveFunctionNonTxContextArgument")]
#[case(
    "receive_bad_args_2",
    "tests/receive/move_sources/receive_bad_args_2.move"
)]
#[should_panic(expected = "ReceiveFunctionIsNotPayable")]
#[case(
    "receive_bad_mutability",
    "tests/receive/move_sources/receive_bad_mutability.move"
)]
fn test_receive_bad(#[case] module_name: &str, #[case] source_path: &'static str) {
    translate_test_package(source_path, module_name);
}
