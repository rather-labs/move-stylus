module test::hash_type_and_key;

const ADDRESS: address = @0xcafecafecafecafecafecafecafecafecafecafe;

// Redefine the funtion from the framwork to be able to use it since
// it is declared as `public(package)`
native fun hash_type_and_key<K: copy + drop + store>(
    parent: address,
    k: K,
): address;

// This function is used to return from test functions the point where
// we want to start reading memory to check if that what we are hasing
// is correct.
native fun get_last_memory_position(): u32;

public fun hash_u8(a: u8): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

public fun hash_u16(a: u16): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

public fun hash_u32(a: u32): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

/*
public fun hash_u64(a: u64): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}
*/

public fun hash_u128(a: u128): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

public fun hash_u256(a: u256): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

public fun hash_bool(a: bool): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

public fun hash_address(a: address): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}
