module test::tx_context;

// use sui::tx_context::TxContext;
use stylus::tx_context::{sender, TxContext};

public fun get_sender(ctx: &TxContext): address {
    ctx.sender()
}
