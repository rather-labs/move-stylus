use common::translate_test_package;

mod common;

/// This test is here to check if code that use the standard library gets compiled to Move
/// Bytecode.
/// We can't translate it to WASM yet so it should panic!
#[test]
#[should_panic(expected = "Unsupported signature token: Struct(DatatypeHandleIndex(0))")]
fn test_use_stdlib() {
    const MODULE_NAME: &str = "use_stdlib";
    const SOURCE_PATH: &str = "tests/stdlib/use_stdlib.move";

    translate_test_package(SOURCE_PATH, MODULE_NAME);
}
