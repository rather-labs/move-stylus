module hello_world::other_mod;

use hello_world::another_mod::{AnotherTest, generic_identity_2};

public struct Test(u8, AnotherTest)

public fun get_test_values(self: &Test): (u8, u8) {
    let Test(value, another_test) = self;
    (*value, another_test.get_another_test_value())
}

public fun generic_identity<T>(t: T): T {
     generic_identity_2(t)
}

public fun generic_identity_two_types<T, U>(t: T, u: U): (T, U) {
    (
        generic_identity(t),
        generic_identity(u),
    )
}

