module test::another_mod;

public struct AnotherTest {
    at_field: u8
}

public fun get_value(self: &AnotherTest): u8 {
   self.at_field
}


