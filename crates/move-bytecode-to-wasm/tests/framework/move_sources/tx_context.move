module test::tx_context;

use stylus::tx_context::TxContext;

entry fun get_sender(ctx: &TxContext): address {
    ctx.sender()
}

entry fun get_msg_value(ctx: &TxContext): u256 {
    ctx.value()
}

entry fun get_block_number(ctx: &TxContext): u64 {
    ctx.block_number()
}

entry fun get_block_basefee(ctx: &TxContext): u256 {
    ctx.block_basefee()
}

entry fun get_block_gas_limit(ctx: &TxContext): u64 {
    ctx.block_gas_limit()
}

entry fun get_block_timestamp(ctx: &TxContext): u64 {
    ctx.block_timestamp()
}

entry fun get_chain_id(ctx: &TxContext): u64 {
    ctx.chain_id()
}

entry fun get_gas_price(ctx: &TxContext): u256 {
    ctx.gas_price()
}

entry fun get_fresh_object_address(ctx: &mut TxContext): (address, address, address) {
    (
        ctx.fresh_object_address(),
        ctx.fresh_object_address(),
        ctx.fresh_object_address(),
    )

}
