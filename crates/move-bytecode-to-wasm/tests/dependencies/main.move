module 0x0::main;

use 0x0::other_mod::Test;
use 0x0::another_mod::AnotherTest;

public fun test(_ctx: &Test): u8 {
    42
}

public fun test2(_ctx: &AnotherTest): u8 {
    42
}
