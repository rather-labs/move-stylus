//! This module tests the capability pattern.
module test::capability;

use stylus::transfer::{Self};
use stylus::object::{Self};
use stylus::object::UID;
use stylus::tx_context::TxContext;
use stylus::tx_context::{Self};

public struct AdminCap has key { id: UID }

entry fun create(ctx: &mut TxContext) {
    transfer::transfer(
        AdminCap { id: object::new(ctx) },
        tx_context::sender(ctx)
    );
}

entry fun admin_cap_fn(_: &AdminCap ) {}

