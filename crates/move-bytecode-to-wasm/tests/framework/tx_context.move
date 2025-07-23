module test::tx_context;

use stylus::tx_context::TxContext;

public fun get_sender(ctx: &TxContext): address {
    ctx.sender()
}

public fun get_msg_value(ctx: &TxContext): u256 {
    ctx.msg_value()
}

public fun get_block_number(ctx: &TxContext): u64 {
    ctx.block_number()
}

public fun get_block_basefee(ctx: &TxContext): u256 {
    ctx.block_basefee()
}

public fun get_block_gas_limit(ctx: &TxContext): u64 {
    ctx.block_gas_limit()
}
