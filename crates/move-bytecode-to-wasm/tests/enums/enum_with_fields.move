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
    StackInts { x: u8, y: u16, z: u32, w: u64},
    HeapInts { x: u128, y: u256},
}

public fun pack_stack_ints(x: u8, y: u16, z: u32, w: u64): IntergerEnum {
    IntergerEnum::StackInts { x, y, z, w}
}

public fun pack_heap_ints(x: u128, y: u256): IntergerEnum {
    IntergerEnum::HeapInts { x, y}
}

entry fun pack_unpack_stack_ints(x: u8, y: u16, z: u32, w: u64): (u8, u16, u32, u64) {
    let integer_enum = pack_stack_ints(x, y, z, w);
    match (integer_enum) {
        IntergerEnum::StackInts { x, y, z, w} => (x, y, z, w),
        _ => {             
            abort(1)
        },
    }
}

entry fun pack_unpack_heap_ints(x: u128, y: u256): (u128, u256) {
    let integer_enum = pack_heap_ints(x, y);
    match (integer_enum) {
        IntergerEnum::HeapInts { x, y} => (x, y),
        _ => {
            abort(1)
        },
    }
}