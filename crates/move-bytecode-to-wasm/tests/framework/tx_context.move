module test::tx_context;

use stylus::tx_context::TxContext;

public fun get_sender(ctx: &TxContext): address {
    ctx.sender()
}
