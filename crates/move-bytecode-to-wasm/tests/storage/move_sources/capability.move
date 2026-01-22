//! This module tests the capability pattern.
module test::capability;

use stylus::{transfer::{Self}, object::{Self, UID}, tx_context::{Self, TxContext}};

public struct AdminCap has key { id: UID }

entry fun create(ctx: &mut TxContext) {
    transfer::transfer(
        AdminCap { id: object::new(ctx) },
        tx_context::sender(ctx)
    );
}

entry fun admin_cap_fn(_: &AdminCap ) {}

