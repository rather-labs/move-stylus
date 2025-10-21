module 0x00::enum_with_fields;

public enum Planets has drop {
    Earth(u64, u64),
    Jupiter(u64, u64),
    Mars(u64, u64),
    Venus(u64, u64),
    Saturn(u64, u64),
}

// Pack and then immediately unpack a given planet.
entry fun pack_unpack_planet(planet_index: u8): (u64, u64) {
    // Pack a Planet variant via PackVariant
    let planet = match(planet_index) {
        0 => Planets::Earth(6371, 5972),
        1 => Planets::Jupiter(69911, 1898000),
        2 => Planets::Mars(3389, 641),
        3 => Planets::Venus(6051, 4868),
        4 => Planets::Saturn(58232, 56800),
        _ => {
            abort
        },
    };

    // Match and unpack a planet variant via UnpackVariant and UnpackVariantImmRef
    // Each arm emits:
    // MoveLoc[i](&Planets)
    // UnpackVariantImmRef(VariantHandleIndex(k))
    // Pop
    // Pop
    // MoveLoc[j](Planets)
    // UnpackVariant(VariantHandleIndex(k))
    // ...
    match (planet) {
        Planets::Earth(radius, mass) => (radius, mass),
        Planets::Jupiter(radius, mass) => (radius, mass),
        Planets::Mars(radius, mass) => (radius, mass),
        Planets::Venus(radius, mass) => (radius, mass),
        Planets::Saturn(radius, mass) => (radius, mass),
    }
}

// Enums to test packing of all possible integer types
public enum IntergerEnum has drop {
    StackInts { x: u8, y: u16, z: u32, w: u64 },
    HeapInts { x: u128, y: u256 },
}

public fun pack_stack_ints(x: u8, y: u16, z: u32, w: u64): IntergerEnum {
    IntergerEnum::StackInts { x, y, z, w }
}

public fun pack_heap_ints(x: u128, y: u256): IntergerEnum {
    IntergerEnum::HeapInts { x, y }
}

entry fun pack_unpack_stack_ints(x: u8, y: u16, z: u32, w: u64): (u8, u16, u32, u64) {
    let integer_enum = pack_stack_ints(x, y, z, w);
    match (integer_enum) {
        IntergerEnum::StackInts { x, y, z, w } => (x, y, z, w),
        _ => {             
            abort(1)
        },
    }
}

entry fun pack_unpack_heap_ints(x: u128, y: u256): (u128, u256) {
    let integer_enum = pack_heap_ints(x, y);
    match (integer_enum) {
        IntergerEnum::HeapInts { x, y } => (x, y),
        _ => {
            abort(1)
        },
    }
}

// Test packvariant/unpackvariant of vectors of integers
public enum IntegerVectorEnum has drop {
    PositionalVectors(vector<u8>, vector<u16>, vector<u32>, vector<u64>),
    NamedVectors{ x: vector<u128>, y: vector<u256> },
    PositionalNestedVectors(vector<vector<u32>>, vector<vector<u64>>)
}

fun pack_positional_vectors(a: u8, b: u16, c: u32, d: u64): IntegerVectorEnum {
    let v8 = vector[a, a+1, a+2];
    let v16 = vector[b, b+1, b+2];
    let v32 = vector[c, c+1, c+2];
    let v64 = vector[d, d+1, d+2];
    let v = IntegerVectorEnum::PositionalVectors(v8, v16, v32, v64);
    v
}

fun pack_named_vectors(x: u128, y: u256): IntegerVectorEnum {
    let vx = vector[x, x+1, x+2];
    let vy = vector[y, y+1, y+2];
    let v = IntegerVectorEnum::NamedVectors{ x: vx, y: vy };
    v
}

fun pack_positional_nested_vectors(x: u32, y: u64): IntegerVectorEnum {
    let vx = vector[vector[x, x+1, x+2], vector[x+3, x+4, x+5]];
    let vy = vector[vector[y, y+1, y+2], vector[y+3, y+4, y+5]];
    let v = IntegerVectorEnum::PositionalNestedVectors(vx, vy);
    v
}

entry fun pack_unpack_positional_vector(a: u8, b: u16, c: u32, d: u64): (vector<u8>, vector<u16>, vector<u32>, vector<u64>) {
    let vec_enum = pack_positional_vectors(a, b, c, d);
    match (vec_enum) {
        IntegerVectorEnum::PositionalVectors(v8, v16, v32, v64) => (v8, v16, v32, v64),
        _ => {             
            abort(1)
        },
    }
}

entry fun pack_unpack_named_vectors(x: u128, y: u256): (vector<u128>, vector<u256>) {
    let vec_enum = pack_named_vectors(x, y);
    match (vec_enum) {
        IntegerVectorEnum::NamedVectors{ x, y } => (x, y),
        _ => {
            abort(1)
        },
    }
}

entry fun pack_unpack_positional_nested_vectors(x: u32, y: u64): (vector<vector<u32>>, vector<vector<u64>>) {
    let vec_enum = pack_positional_nested_vectors(x, y);
    match (vec_enum) {
        IntegerVectorEnum::PositionalNestedVectors(vx, vy) => (vx, vy),
        _ => {
            abort(1)
        },
    }
}

// Enum with structs as fields
public struct Alpha has drop {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
}

public struct Beta has store, drop {
    e: u128,
    f: u256,
}

public struct Gamma has drop {
    a: vector<u32>,
    b: vector<bool>,
    c: Beta
}

public enum StructsEnum has drop {
    Alpha(Alpha),
    Beta { beta: Beta },
    Gamma { gamma: Gamma },
}

fun pack_alpha(a: u8, b: u16, c: u32, d: u64): StructsEnum {
    let alpha = Alpha { a, b, c, d };
    StructsEnum::Alpha(alpha)
}

fun pack_beta(e: u128, f: u256): StructsEnum {
    let beta = Beta { e, f };
    StructsEnum::Beta { beta }
}

fun pack_gamma(a: vector<u32>, b: vector<bool>, c: u128, d: u256): StructsEnum {
    let beta = Beta { e: c, f: d };
    let gamma = Gamma { a, b, c: beta };
    StructsEnum::Gamma { gamma }
}

entry fun pack_unpack_alpha(a: u8, b: u16, c: u32, d: u64): (u8, u16, u32, u64) {
    let structs_enum = pack_alpha(a, b, c, d);
    match (structs_enum) {
        StructsEnum::Alpha(alpha) => (alpha.a, alpha.b, alpha.c, alpha.d),
        _ => {
            abort(1)
        },
    }
}

entry fun pack_unpack_beta(e: u128, f: u256): (u128, u256) {
    let structs_enum = pack_beta(e, f);
    match (structs_enum) {
        StructsEnum::Beta { beta } => (beta.e, beta.f),
        _ => {
            abort(1)
        },
    }
}

entry fun pack_unpack_gamma(a: vector<u32>, b: vector<bool>, c: u128, d: u256): (vector<u32>, vector<bool>, u128, u256) {
    let structs_enum = pack_gamma(a, b, c, d);
    match (structs_enum) {
        StructsEnum::Gamma { gamma } => (gamma.a, gamma.b, gamma.c.e, gamma.c.f),
        _ => {
            abort(1)
        },
    }
}

entry fun get_gamma_vec_sum(a: vector<u32>, b: vector<bool>, c: u128, d: u256): u32 {
    let structs_enum = pack_gamma(a, b, c, d);
    match (structs_enum) {
        StructsEnum::Gamma { gamma } => { 
            let mut vec_sum = 0u32;
            let mut i = 0;
            while (i < vector::length(&gamma.a)) {
                vec_sum = vec_sum + *vector::borrow(&gamma.a, i);
                i = i + 1;
            };
            vec_sum
        },
        _ => {
            abort(1)
        },
    }
}