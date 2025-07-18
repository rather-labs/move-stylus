module hello_world::hello_world;

// use sui::tx_context::TxContext;
use sui::tx_context::{sender, TxContext};
use hello_world::other_mod::Test;
use hello_world::another_mod::AnotherTest;

//  TODO: Add support for native functions
//  native public fun emit_log(ptr: u32, len: u32, topic: u32);

public fun test(ctx: &TxContext): address {
    ctx.sender()
    // ctx.sender
}

/*
public fun test(): address {
    sender()
    // ctx.sender
}*/
