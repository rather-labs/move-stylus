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

// Dynamic fields

public struct DynamicStruct has key {
    id: UID,
    a: u32,
    b: bool,
    c: vector<u64>,
    d: vector<u128>,
    e: u64,
    f: u128,
    g: u256,
}

public struct DynamicStruct2 has key {
    id: UID,
    a: vector<bool>,
    b: vector<u8>,
    c: vector<u16>,
    d: vector<u32>,
    e: vector<u64>,
    f: vector<u128>,
    g: vector<u256>,
    h: vector<address>,
}

public struct DynamicStruct3 has key {
    id: UID,
    a: vector<vector<u8>>,
    b: vector<vector<u32>>,
    c: vector<vector<u64>>,
    d: vector<vector<u128>>,
}

public struct DynamicNestedStructChild has store {
    a: vector<u32>,
    b: u128
}

public struct DynamicStruct4 has key {
    id: UID,
    a: vector<DynamicNestedStructChild>,
    b: vector<StaticNestedStructChild>,
}

// public struct DynamicStruct6 has key {
//     id: UID,
//     c: vector<vector<StaticNestedStructChild>>,
//     d: vector<u64>,
// }

public fun save_dynamic_struct(
    id: UID,
    a: u32,
    b: bool,
    c: vector<u64>,
    d: vector<u128>,
    e: u64,
    f: u128,
    g: u256,
) {
    let struct_ = DynamicStruct { id, a, b, c, d, e, f, g };
    save_in_slot(struct_, 0);
}

public fun read_dynamic_struct(): DynamicStruct {
    read_slot<DynamicStruct>(0)
}

public fun save_dynamic_struct_2(
    id: UID,
    a: vector<bool>,
    b: vector<u8>,
    c: vector<u16>,
    d: vector<u32>,
    e: vector<u64>,
    f: vector<u128>,
    g: vector<u256>,
    h: vector<address>,
) { 
    let struct_ = DynamicStruct2 { id, a, b, c, d, e, f, g, h };
    save_in_slot(struct_, 0);
}

public fun read_dynamic_struct_2(): DynamicStruct2 {
    read_slot<DynamicStruct2>(0)
}

public fun save_dynamic_struct_3(
    id: UID,
    a: vector<vector<u8>>,
    b: vector<vector<u32>>,
    c: vector<vector<u64>>,
    d: vector<vector<u128>>,
) {
    let struct_ = DynamicStruct3 { id, a, b, c, d };
    save_in_slot(struct_, 0);
}

public fun read_dynamic_struct_3(): DynamicStruct3 {
    read_slot<DynamicStruct3>(0)
}

public fun save_dynamic_struct_4(
    id: UID,
    x: vector<u32>,
    y: u64,
    z: u128,
    w: address
) {
    let a = vector[DynamicNestedStructChild { a: x, b: z }, DynamicNestedStructChild { a: x, b: z + 1 }];
    let b = vector[StaticNestedStructChild { d: y, e: w }, StaticNestedStructChild { d: y + 1 , e: w }, StaticNestedStructChild { d: y + 2, e: w }];
    let struct_ = DynamicStruct4 { id, a, b};
    save_in_slot(struct_, 0);
}

public fun read_dynamic_struct_4(): DynamicStruct4 {
    read_slot<DynamicStruct4>(0)
}

// public fun save_dynamic_struct_6(
//     id: UID,
//     a: u64,
//     b: u128
// ) {
//     let c = vector[vector[StaticNestedStructChild { a, b }, StaticNestedStructChild { a, b }], vector[StaticNestedStructChild { a, b }, StaticNestedStructChild { a, b }]];
//     let d = vector[a, a + 1, a + 2];
//     let struct_ = DynamicStruct6 { id, c, d};
//     save_in_slot(struct_, 0);
// }

// public fun read_dynamic_struct_6(): DynamicStruct6 {
//     read_slot<DynamicStruct6>(0)
// }