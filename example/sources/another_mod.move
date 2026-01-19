module hello_world::another_mod;

use stylus::object::{ID, UID};
use stylus::object as object;

public struct AnotherTest(u8)

public struct AnotherTest2(ID)

public entry fun create_another_test(x: u8): AnotherTest {
    AnotherTest(x)
}

public entry fun create_another_test_2(t: &UID): AnotherTest2 {
    AnotherTest2(t.to_inner())
}

public entry fun get_another_test_value(self: &AnotherTest): u8 {
    let AnotherTest(value) = self;
    *value
}

public fun generic_identity_2<T>(t: T): T {
    t
}


