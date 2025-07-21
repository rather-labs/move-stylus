module test::main;

use test::other_mod::Test;
use test::another_mod::AnotherTest;

public fun echo_test_struct(test: &Test): (u8, u8) {
    let (a, b) = test.get_values();
    (a, b)
}

public fun echo_another_test_struct(another_test: &AnotherTest): u8 {
    another_test.get_value()
}
