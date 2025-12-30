module 0x00::enums_magical_creatures;

const E_NOT_SPIRIT: u64 = 1;
const E_NOT_GOLEM: u64 = 2;

/// Beast: primal creatures with raw power and instinct
public struct Beast has drop {
    level: u32,
    ferocity: u64,
}

/// Golem: ancient constructs powered by elemental energy
public struct Golem has drop {
    level: u32,
    density: u128,
    shards: vector<u64>,
}

/// Spirit: ethereal beings of pure magical essence
public struct Spirit has drop {
    level: u32,
    age: u64,
    chants: vector<vector<u8>>, // "spells" as string-like bytes
}

public enum Creature has drop {
    BeastVariant(Beast),
    GolemVariant(Golem),
    SpiritVariant(Spirit),
}

// Utility functions for working with creatures
fun get_creature_level(creature: &Creature): u32 {
    match (creature) {
        Creature::BeastVariant(beast) => beast.level,
        Creature::GolemVariant(golem) => golem.level,
        Creature::SpiritVariant(spirit) => spirit.level,
    }
}

fun get_creature_power_rating(creature: &Creature): u64 {
    match (creature) {
        Creature::BeastVariant(beast) => (beast.level as u64) * (beast.ferocity as u64),
        Creature::GolemVariant(golem) => {
            let mut i = 0;
            let mut shard_power = 0u64;
            while (i < vector::length(&golem.shards)) {
                shard_power = shard_power + *vector::borrow(&golem.shards, i);
                i = i + 1;
            };

            (golem.density as u64) + (golem.level as u64) + shard_power
        },
        Creature::SpiritVariant(spirit) => {
            let mut i = 0;
            let mut chant_power = 0u64;
            while (i < vector::length(&spirit.chants)) {
                let chant = *vector::borrow(&spirit.chants, i);
                chant_power = chant_power + vector::length(&chant);
                i = i + 1;
            };
            spirit.age + (spirit.level as u64) + chant_power
        },
    }
}

fun add_chant(creature: &mut Creature, chant: vector<u8>) {
    match (creature) {
        Creature::SpiritVariant(spirit) => vector::push_back(&mut spirit.chants, chant),
        _ => abort(E_NOT_SPIRIT),
    }
}

fun add_shard(creature: &mut Creature, shard: u64) {
    match (creature) {
        Creature::GolemVariant(golem) => vector::push_back(&mut golem.shards, shard),
        _ => abort(E_NOT_GOLEM),
    }
}

fun increase_level(creature: &mut Creature) {
    match (creature) {
        Creature::BeastVariant(beast) => beast.level = beast.level + 1,
        Creature::GolemVariant(golem) => golem.level = golem.level + 1,
        Creature::SpiritVariant(spirit) => spirit.level = spirit.level + 1,
    }
}

// Constructor functions for easier creature creation
fun create_beast(level: u32, ferocity: u64): Beast {
    Beast {
        level,
        ferocity,
    }
}

fun create_golem(level: u32, density: u128, shards: vector<u64>): Golem {
    let _v = vector[vector[false], vector[true], vector[false], vector[true]];
    Golem {
        level,
        density,
        shards,
    }
}

fun create_spirit(level: u32, chants: vector<vector<u8>>, age: u64): Spirit {
    Spirit {
        level,
        chants,
        age,
    }
}

entry fun test_beast(level: u32, ferocity: u64): (u32, u64, u32, u64) {
    let beast = create_beast(level, ferocity);
    let mut beast_variant = Creature::BeastVariant(beast);
    let beast_level_0 = get_creature_level(&beast_variant);
    let beast_power_0 = get_creature_power_rating(&beast_variant);
    increase_level(&mut beast_variant);
    let beast_level_1 = get_creature_level(&beast_variant);
    let beast_power_1 = get_creature_power_rating(&beast_variant);
    (beast_level_0, beast_power_0, beast_level_1, beast_power_1)
}

entry fun test_golem(level: u32, density: u128, shards: vector<u64>): (u32, u64, u32, u64) {
    let golem = create_golem(level, density, shards);
    let mut golem_variant = Creature::GolemVariant(golem);
    let golem_level_0 = get_creature_level(&golem_variant);
    let golem_power_0 = get_creature_power_rating(&golem_variant);
    add_shard(&mut golem_variant, 5);
    increase_level(&mut golem_variant);
    add_shard(&mut golem_variant, 10);
    let golem_level_1 = get_creature_level(&golem_variant);
    let golem_power_1 = get_creature_power_rating(&golem_variant);
    (golem_level_0, golem_power_0, golem_level_1, golem_power_1)
}

entry fun test_spirit(level: u32, chants: vector<vector<u8>>, age: u64): (u32, u64, u32, u64) {
    let spirit = create_spirit(level, chants, age);
    let mut spirit_variant = Creature::SpiritVariant(spirit);
    let spirit_level_0 = get_creature_level(&spirit_variant);
    let spirit_power_0 = get_creature_power_rating(&spirit_variant);
    add_chant(&mut spirit_variant, vector[4, 4, 3, 1]);
    increase_level(&mut spirit_variant);
    add_chant(&mut spirit_variant, vector[8, 95]);
    let spirit_level_1 = get_creature_level(&spirit_variant);
    let spirit_power_1 = get_creature_power_rating(&spirit_variant);
    (spirit_level_0, spirit_power_0, spirit_level_1, spirit_power_1)
}
