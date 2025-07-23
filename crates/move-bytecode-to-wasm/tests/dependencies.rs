use common::translate_test_complete_package;

mod common;

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
#[test]
fn test_dependencies() {
    translate_test_complete_package("tests/dependencies");
}
