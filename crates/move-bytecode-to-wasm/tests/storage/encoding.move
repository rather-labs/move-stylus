module test::storage_encoding;

use stylus::object::UID;
use stylus::tx_context::TxContext;
use stylus::object;

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
    a: u256,
    b: u128,
    c: u64,
    d: u32,
    e: u16,
    f: u8,
    g: address,
    ctx: &mut TxContext
) {
    let struct_ = StaticFields { id: object::new(ctx), a, b, c, d, e, f, g };
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
    a: u8,
    b: address,
    c: u64,
    d: u16,
    e: u8,
    ctx: &mut TxContext
) {
    let struct_ = StaticFields2 { id: object::new(ctx), a, b, c, d, e };
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
    a: u8,
    b: address,
    c: u64,
    d: address,
    ctx: &mut TxContext
) {
    let struct_ = StaticFields3 { id: object::new(ctx), a, b, c, d };
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
    a: u64,
    b: bool,
    d: u64,
    e: address,
    f: u128,
    g: u32, 
    ctx: &mut TxContext
) {
    let child = StaticNestedStructChild { d, e };
    let struct_ = StaticNestedStruct { id: object::new(ctx), a, b, c: child, f, g };
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

public struct NestedStructChildWrapper has store {
    a: vector<DynamicNestedStructChild>,
    b: vector<StaticNestedStructChild>
}

public struct DynamicStruct5 has key {
    id: UID,
    a: vector<NestedStructChildWrapper>,
}

public struct GenericStruct<T> has key {
    id: UID,
    a: vector<T>,
    b: T,
}

public fun save_dynamic_struct(
    a: u32,
    b: bool,
    c: vector<u64>,
    d: vector<u128>,
    e: u64,
    f: u128,
    g: u256,
    ctx: &mut TxContext
) {
    let struct_ = DynamicStruct { id: object::new(ctx), a, b, c, d, e, f, g };
    save_in_slot(struct_, 0);
}

public fun read_dynamic_struct(): DynamicStruct {
    read_slot<DynamicStruct>(0)
}

public fun save_dynamic_struct_2(
    a: vector<bool>,
    b: vector<u8>,
    c: vector<u16>,
    d: vector<u32>,
    e: vector<u64>,
    f: vector<u128>,
    g: vector<u256>,
    h: vector<address>,
    ctx: &mut TxContext
) {
    let struct_ = DynamicStruct2 { id: object::new(ctx), a, b, c, d, e, f, g, h };
    save_in_slot(struct_, 0);
}

public fun read_dynamic_struct_2(): DynamicStruct2 {
    read_slot<DynamicStruct2>(0)
}

public fun save_dynamic_struct_3(
    a: vector<vector<u8>>,
    b: vector<vector<u32>>,
    c: vector<vector<u64>>,
    d: vector<vector<u128>>,
    ctx: &mut TxContext
) {
    let struct_ = DynamicStruct3 { id: object::new(ctx), a, b, c, d };
    save_in_slot(struct_, 0);
}

public fun read_dynamic_struct_3(): DynamicStruct3 {
    read_slot<DynamicStruct3>(0)
}

public fun save_dynamic_struct_4(
    x: vector<u32>,
    y: u64,
    z: u128,
    w: address,
    ctx: &mut TxContext
) {
    let a = vector[DynamicNestedStructChild { a: x, b: z }, DynamicNestedStructChild { a: x, b: z + 1 }];
    let b = vector[StaticNestedStructChild { d: y, e: w }, StaticNestedStructChild { d: y + 1 , e: w }, StaticNestedStructChild { d: y + 2, e: w }];
    let struct_ = DynamicStruct4 { id: object::new(ctx), a, b};
    save_in_slot(struct_, 0);
}

public fun read_dynamic_struct_4(): DynamicStruct4 {
    read_slot<DynamicStruct4>(0)
}

public fun save_dynamic_struct_5(
    x: u32,
    y: u64,
    z: u128,
    w: address,
    ctx: &mut TxContext
) {
    let v = vector[x, x + 1, x + 2];
    let a1 = vector[DynamicNestedStructChild { a: v, b: z }, DynamicNestedStructChild { a: v, b: z + 1 }];
    let a2 = vector[DynamicNestedStructChild { a: v, b: z + 2 }, DynamicNestedStructChild { a: v, b: z + 3 }, DynamicNestedStructChild { a: v, b: z + 4 }];
    let b1 = vector[StaticNestedStructChild { d: y, e: w }, StaticNestedStructChild { d: y + 1 , e: w }, StaticNestedStructChild { d: y + 2, e: w }];
    let b2 = vector[StaticNestedStructChild { d: y + 3, e: w }, StaticNestedStructChild { d: y + 4 , e: w }];
    let a = vector[NestedStructChildWrapper { a: a1, b: b1 }, NestedStructChildWrapper { a: a2, b: b2 }];
    let struct_ = DynamicStruct5 { id: object::new(ctx), a};
    save_in_slot(struct_, 0);
}

public fun read_dynamic_struct_5(): DynamicStruct5 {
    read_slot<DynamicStruct5>(0)
}

public fun save_generic_struct_32(
    x: u32,
    ctx: &mut TxContext
) {
    let a = vector[x, x + 1, x + 2];
    let struct_ = GenericStruct<u32> { id: object::new(ctx), a, b: x };
    save_in_slot(struct_, 0);
}

public fun read_generic_struct_32(): GenericStruct<u32> {
    read_slot<GenericStruct<u32>>(0)
}

/// Structs with wrapped objects fields

// Simple value struct with key
public struct Bar has key, store {
    id: UID,
    a: u64,
}

// Struct with nested field struct with key
public struct Foo has key, store {
    id: UID,
    a: u64,
    b: Bar,
    c: u32
}

public fun save_foo(ctx: &mut TxContext) {
    let bar = Bar {
        id: object::new(ctx),
        a: 42,
    };

    let foo = Foo {
        id: object::new(ctx),
        a: 101,
        b: bar,
        c: 102,
    };

    save_in_slot(foo, 0);
}

public fun read_foo(): Foo {
    read_slot<Foo>(0)
}

public struct MegaFoo has key {
    id: UID,
    a: u64,
    b: Foo,
    c: u32
}

public fun save_mega_foo(ctx: &mut TxContext) {
    let bar = Bar {
        id: object::new(ctx),
        a: 42,
    };

    let foo = Foo {
        id: object::new(ctx),
        a: 101,
        b: bar,
        c: 102,
    };

    let mega_foo = MegaFoo {
        id: object::new(ctx),
        a: 77,
        b: foo,
        c: 88,
    };

    save_in_slot(mega_foo, 0);
}

public fun read_mega_foo(): MegaFoo {
    read_slot<MegaFoo>(0)
}