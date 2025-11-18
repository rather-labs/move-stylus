mod common;

use common::test_generated_abi;
use rstest::rstest;

#[rstest]
#[case("simple", "modules/simple.move", "json_format/simple.json")]
#[case(
    "abi_error_1",
    "modules/abi_errors/abi_error_1.move",
    "json_format/abi_errors/abi_error_1.json"
)]
#[case(
    "abi_error_2",
    "modules/abi_errors/abi_error_2.move",
    "json_format/abi_errors/abi_error_2.json"
)]
#[case(
    "abi_error_3",
    "modules/abi_errors/abi_error_3.move",
    "json_format/abi_errors/abi_error_3.json"
)]
fn test_abi_generation(
    #[case] module_name: &str,
    #[case] module_path: &str,
    #[case] json_path: &str,
) {
    let module_path = format!("tests/{module_path}");
    let json_path = format!("tests/{json_path}");
    test_generated_abi(&json_path, &module_path, module_name).unwrap();
}
