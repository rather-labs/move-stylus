module test::misc_external;

use stylus::tx_context::TxContext;
use stylus::object::{Self};
use stylus::object::UID;

public struct ExternalKeyStruct has key {
    id: UID,
    owner: address,
    value: u64
}

public fun new_external_key_struct(value: u64, ctx: &mut TxContext): ExternalKeyStruct {
    return (ExternalKeyStruct { id: object::new(ctx), owner: ctx.sender(), value })
}