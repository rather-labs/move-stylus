module test::erc20;

use stylus::event::emit;
use std::ascii::String;
use std::ascii as ascii;
use stylus::tx_context::TxContext;
use stylus::transfer as transfer;
use stylus::object as object;
use stylus::object::NamedId;
use stylus::object::UID;
use stylus::dynamic_field_named_id as field;
use stylus::table::Table;
use stylus::table as table;


public struct ALLOWANCE_ has key {}

public struct Allowance has key {
    id: NamedId<ALLOWANCE_>,
}

public fun create(ctx: &mut TxContext) {
    transfer::share_object(Allowance {
        id: object::new_named_id<ALLOWANCE_>(),
    });
}

public fun approve(
    spender: address,
    amount: u256,
    allowance: &mut Allowance,
    ctx: &mut TxContext,
): bool {
        field::add(
            &mut allowance.id,
            ctx.sender(),
            table::new<address, u256>(ctx)
        );
        true
}
