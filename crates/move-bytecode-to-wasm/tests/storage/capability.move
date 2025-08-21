//! This module tests the capability pattern.
module test::capability;

use stylus::transfer as transfer;
use stylus::object as object;
use stylus::object::UID;
use stylus::tx_context::TxContext;
use stylus::tx_context as tx_context;

public struct AdminCap has key { id: UID }

public fun create(ctx: &mut TxContext) {
    transfer::transfer(
        AdminCap { id: object::new(ctx) },
        tx_context::sender(ctx)
    );
}

public fun admin_cap_fn(_: &AdminCap ) {}

