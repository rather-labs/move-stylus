module hello_world::another_mod;

public struct AnotherTest(u8)

public fun create_another_test(x: u8): AnotherTest {
    AnotherTest(x)
}

public fun get_another_test_value(self: &AnotherTest): u8 {
    let AnotherTest(value) = self;
    *value
}

public fun generic_identity_2<T>(t: T): T {
    t
}


