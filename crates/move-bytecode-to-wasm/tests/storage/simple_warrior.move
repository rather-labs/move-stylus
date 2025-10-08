// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Demonstrates wrapping objects using the `Option` type.
module test::simple_warrior;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct Warrior has key {
    id: UID,
    sword: Option<Sword>,
    shield: Option<Shield>,
}

public struct Sword has key, store {
    id: UID,
    strength: u8,
}

public struct Shield has key, store {
    id: UID,
    armor: u8,
}

public fun create_warrior(ctx: &mut TxContext) {
    let warrior = Warrior {
        id: object::new(ctx),
        sword: option::none(),
        shield: option::none(),
    };
    transfer::transfer(warrior, ctx.sender())
}

public fun create_sword(strength: u8, ctx: &mut TxContext) {
    let sword = Sword { id: object::new(ctx), strength: strength };
    transfer::transfer(sword, ctx.sender())
}

public fun create_shield(armor: u8, ctx: &mut TxContext) {
    let shield = Shield { id: object::new(ctx), armor: armor };
    transfer::transfer(shield, ctx.sender())
}

public fun equip_sword(warrior: &mut Warrior, sword: Sword, ctx: &mut TxContext) {
    if (warrior.sword.is_some()) {
        let old_sword = warrior.sword.extract();
        transfer::transfer(old_sword, ctx.sender());
    };
    warrior.sword.fill(sword);
}

public fun equip_shield(warrior: &mut Warrior, shield: Shield, ctx: &mut TxContext) {
    if (warrior.shield.is_some()) {
        let old_shield = warrior.shield.extract();
        transfer::transfer(old_shield, ctx.sender());
    };
    warrior.shield.fill(shield);
}

public fun destroy_warrior(warrior: Warrior) {
    let Warrior { id, sword: mut sword, shield: mut shield } = warrior;

    // delete the Warrior UID first
    object::delete(id);

    // --- Sword ---
    if (option::is_some(&sword)) {
        // extract consumes the inner Sword and leaves sword == None
        let s = option::extract(&mut sword);
        let Sword { id, strength: _ } = s;
        object::delete(id);
    };
    // sword is None now (either originally or after extract); consume it
    option::destroy_none(sword);

    // --- Shield ---
    if (option::is_some(&shield)) {
        let sh = option::extract(&mut shield);
        let Shield { id, armor: _ } = sh;
        object::delete(id);
    };
    option::destroy_none(shield);
}

public fun destroy_sword(sword: Sword) {
    let Sword { id, strength: _ } = sword;
    object::delete(id);
}

public fun destroy_shield(shield: Shield) {
    let Shield { id, armor: _ } = shield;
    object::delete(id);
}

public fun inspect_warrior(warrior: &Warrior): &Warrior {
    warrior
}

public fun inspect_sword(sword: &Sword): &Sword {
    sword
}

public fun inspect_shield(shield: &Shield): &Shield {
    shield
}