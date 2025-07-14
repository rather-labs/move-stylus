module 0x01::hello_world;

use sui::tx_context::TxContext;

public fun test(ctx: &TxContext): u8 {
    42
}
