module test::storage_encoding;

use stylus::object::UID;

// This function will facilitate the reading from the test.
native fun save_in_slot<T: key>(value: T, slot: u256);

public struct StaticFields has key {
    id: UID,
    a: u256,
    b: u128,
    c: u64,
    d: u32,
    e: u16,
    f: u8,
    g: address,
}

public fun save_static_fields(
    id: UID,
    a: u256,
    b: u128,
    c: u64,
    d: u32,
    e: u16,
    f: u8,
    g: address
) {
    let struct_ = StaticFields { id, a, b, c, d, e, f, g };
    save_in_slot(struct_, 0);
}

public struct StaticFields2 has key {
    id: UID,
    a: u8,
    b: address,
    c: u64,
    d: u16,
    e: u8,
}

public fun save_static_fields_2(
    id: UID,
    a: u8,
    b: address,
    c: u64,
    d: u16,
    e: u8
) {
    let struct_ = StaticFields2 { id, a, b, c, d, e };
    save_in_slot(struct_, 0);
}

public struct StaticFields3 has key {
    id: UID,
    a: u8,
    b: address,
    c: u64,
    d: address,
}

public fun save_static_fields_3(
    id: UID,
    a: u8,
    b: address,
    c: u64,
    d: address
) {
    let struct_ = StaticFields3 { id, a, b, c, d };
    save_in_slot(struct_, 0);
}
