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

public struct Bar has copy, drop, store {
    n: u32,
    o: u128,
}

public struct Foo has copy, drop, store {
    p: Bar,
    q: address,
    r: vector<u32>,
    s: vector<u128>,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
}

public struct Baz<T: copy + drop + store> has copy, drop, store {
    g: T,
    p: Bar,
    q: address,
    r: vector<u32>,
    s: vector<u128>,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
}

entry fun hash_u8(a: u8): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_u16(a: u16): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_u32(a: u32): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_u64(a: u64): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_u128(a: u128): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_u256(a: u256): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_bool(a: bool): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_address(a: address): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}


entry fun hash_vector_u8(a: vector<u8>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_vector_u16(a: vector<u16>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_vector_u32(a: vector<u32>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_vector_u64(a: vector<u64>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_vector_u128(a: vector<u128>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_vector_u256(a: vector<u256>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_vector_bool(a: vector<bool>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_vector_address(a: vector<address>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_bar(a: Bar): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_foo(a: Foo): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_baz_u8(a: Baz<u8>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}

entry fun hash_baz_v_u16(a: Baz<vector<u16>>): u32 {
    hash_type_and_key(ADDRESS, a);
    get_last_memory_position()
}
