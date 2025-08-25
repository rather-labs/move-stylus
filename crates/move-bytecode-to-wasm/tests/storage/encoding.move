module test::storage_encoding;

use stylus::object::UID;

// This function will facilitate the reading from the test.
native fun save_in_slot<T: key>(value: T, slot: u256);
native fun read_slot<T: key>(slot: u256): T;

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

public fun read_static_fields(): StaticFields {
    read_slot<StaticFields>(0)
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

public fun read_static_fields_2(): StaticFields2 {
    read_slot<StaticFields2>(0)
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

public fun read_static_fields_3(): StaticFields3 {
    read_slot<StaticFields3>(0)
}

public struct StaticNestedStruct has key {
    id: UID,
    a: u64,
    b: bool,
    c: StaticNestedStructChild,
    f: u128,
    g: u32,
}

public struct StaticNestedStructChild has store {
    d: u64,
    e: address
}

public fun save_static_nested_struct(
    id: UID,
    a: u64,
    b: bool,
    d: u64,
    e: address,
    f: u128,
    g: u32
) {
    let child = StaticNestedStructChild { d, e };
    let struct_ = StaticNestedStruct { id, a, b, c: child, f, g };
    save_in_slot(struct_, 0);
}

public fun read_static_nested_struct(): StaticNestedStruct {
    read_slot<StaticNestedStruct>(0)
}
