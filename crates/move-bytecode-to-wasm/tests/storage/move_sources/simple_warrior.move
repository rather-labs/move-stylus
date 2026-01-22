// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Demonstrates wrapping objects using the `Option` type.
module test::simple_warrior;

use stylus::tx_context::TxContext;
use stylus::object::{Self};
use stylus::object::UID;
use stylus::transfer::{Self};

public struct Warrior has key {
    id: UID,
    sword: Option<Sword>,
    shield: Option<Shield>,
    faction: Faction,
}

public struct Sword has key, store {
    id: UID,
    strength: u8,
}

public struct Shield has key, store {
    id: UID,
    armor: u8,
}

public enum Faction has drop, store {
    Alliance,
    Horde,
    Rebel
}

entry fun create_warrior(ctx: &mut TxContext) {
    let warrior = Warrior {
        id: object::new(ctx),
        sword: option::none(),
        shield: option::none(),
        faction: Faction::Rebel,
    };
    transfer::transfer(warrior, ctx.sender())
}

entry fun create_sword(strength: u8, ctx: &mut TxContext) {
    let sword = Sword { id: object::new(ctx), strength: strength };
    transfer::transfer(sword, ctx.sender())
}

entry fun create_shield(armor: u8, ctx: &mut TxContext) {
    let shield = Shield { id: object::new(ctx), armor: armor };
    transfer::transfer(shield, ctx.sender())
}

entry fun equip_sword(warrior: &mut Warrior, sword: Sword, ctx: &TxContext) {
    if (warrior.sword.is_some()) {
        let old_sword = warrior.sword.extract();
        transfer::transfer(old_sword, ctx.sender());
    };
    warrior.sword.fill(sword);
}

entry fun equip_shield(warrior: &mut Warrior, shield: Shield, ctx: &TxContext) {
    if (warrior.shield.is_some()) {
        let old_shield = warrior.shield.extract();
        transfer::transfer(old_shield, ctx.sender());
    };
    warrior.shield.fill(shield);
}

entry fun change_faction(warrior: &mut Warrior, faction: Faction) {
    warrior.faction = faction;
}

entry fun destroy_warrior(warrior: Warrior) {
    let Warrior { id, sword: mut sword, shield: mut shield, faction: _ } = warrior;

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

entry fun destroy_sword(sword: Sword) {
    let Sword { id, strength: _ } = sword;
    object::delete(id);
}

entry fun destroy_shield(shield: Shield) {
    let Shield { id, armor: _ } = shield;
    object::delete(id);
}

entry fun inspect_warrior(warrior: &Warrior): &Warrior {
    warrior
}

entry fun inspect_sword(sword: &Sword): &Sword {
    sword
}

entry fun inspect_shield(shield: &Shield): &Shield {
    shield
}
