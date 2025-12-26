use crate::common::translate_test_package;

/// This test is here to check if code that use the standard library gets compiled to Move
/// Bytecode.
#[test]
fn test_use_stdlib() {
    translate_test_package("tests/stdlib/move_sources/use_stdlib.move", "use_stdlib");
}
