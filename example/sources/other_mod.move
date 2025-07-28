module hello_world::other_mod;

use hello_world::another_mod::AnotherTest;

public struct Test(u8, AnotherTest)

public fun get_test_values(self: &Test): (u8, u8) {
    let Test(value, another_test) = self;
    (*value, another_test.get_another_test_value())
}




