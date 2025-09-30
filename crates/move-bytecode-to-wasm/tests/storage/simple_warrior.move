// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Demonstrates wrapping objects using the `Option` type.
module test::simple_warrior;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct Sword has key, store {
    id: UID,
    strength: u8,
}

public struct Warrior has key, store {
    id: UID,
    sword: Option<Sword>,
}

/// Warrior already has a Sword equipped.
const EAlreadyEquipped: u64 = 1;

/// Warrior does not have a sword equipped.
const ENotEquipped: u64 = 2;

public fun new_sword(strength: u8, ctx: &mut TxContext) {
    let sword = Sword { id: object::new(ctx), strength };
    transfer::share_object(sword);
}

public fun new_warrior(ctx: &mut TxContext) {
    let warrior = Warrior { id: object::new(ctx), sword: option::none() };
    transfer::share_object(warrior);
}

public fun equip(warrior: &mut Warrior, sword: Sword) {
    assert!(option::is_none(&warrior.sword), EAlreadyEquipped);
    option::fill(&mut warrior.sword, sword);
}

public fun unequip(warrior: &mut Warrior): Sword {
    assert!(option::is_some(&warrior.sword), ENotEquipped);
    option::extract(&mut warrior.sword)
}