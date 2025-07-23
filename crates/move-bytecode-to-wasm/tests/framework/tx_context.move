module test::tx_context;

use stylus::tx_context::TxContext;

public fun get_sender(ctx: &TxContext): address {
    ctx.sender()
}

public fun get_msg_value(ctx: &TxContext): u256 {
    ctx.msg_value()
}
