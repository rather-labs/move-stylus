module 0x0::other_mod;

use test::another_mod::AnotherTest;

public struct Test {
    t_field_1: u8,
    t_field_2: AnotherTest
}

public fun get_values(self: &Test): (u8, u8) {
    (self.t_field_1, self.t_field_2.get_value())
}


