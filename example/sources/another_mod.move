module hello_world::another_mod;
use stylus::tx_context::{sender, TxContext};

public struct AnotherTest(u8)

// public struct ANOTHER_MOD has drop {}

fun init(ctx: &TxContext) {}