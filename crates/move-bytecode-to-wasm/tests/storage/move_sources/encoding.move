/// Storage Encoding Test Module
///
/// This module provides comprehensive test cases for storage encoding functionality
/// in the Stylus framework. It includes various data structures and operations
/// to test different encoding scenarios including static fields, dynamic fields,
/// nested structures, and object wrapping patterns.

module test::storage_encoding;

use stylus::{
    object::{Self, UID}, 
    tx_context::TxContext
};

// ============================================================================
// Native Functions
// ============================================================================

/// Native functions to facilitate testing storage operations
native fun save_in_slot<T: key>(value: T, slot: u256);
native fun read_slot<T: key>(slot: u256, uid: u256): T;

// ============================================================================
// Static Field Structures
// ============================================================================

/// Test structure with various primitive static fields
#[allow(unused_field)]
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

#[allow(unused_field)]
public struct StaticFields_ {
    id: address,
    a: u256,
    b: u128,
    c: u64,
    d: u32,
    e: u16,
    f: u8,
    g: address,
}

/// Test structure with mixed primitive types
#[allow(unused_field)]
public struct StaticFields2 has key {
    id: UID,
    a: u8,
    b: address,
    c: u64,
    d: u16,
    e: u8,
}

/// Test structure with mixed primitive types
#[allow(unused_field)]
public struct StaticFields2_ {
    id: address,
    a: u8,
    b: address,
    c: u64,
    d: u16,
    e: u8,
}

/// Test structure with address fields
#[allow(unused_field)]
public struct StaticFields3 has key {
    id: UID,
    a: u8,
    b: address,
    c: u64,
    d: address,
}

#[allow(unused_field)]
public struct StaticFields3_ {
    id: address,
    a: u8,
    b: address,
    c: u64,
    d: address,
}

/// Test structure with nested static child
#[allow(unused_field)]
public struct StaticNestedStruct has key {
    id: UID,
    a: u64,
    b: bool,
    c: StaticNestedStructChild,
    f: u128,
    g: u32,
}

#[allow(unused_field)]
public struct StaticNestedStruct_ {
    id: address,
    a: u64,
    b: bool,
    c: StaticNestedStructChild,
    f: u128,
    g: u32,
}

/// Child structure for static nested testing
#[allow(unused_field)]
public struct StaticNestedStructChild has store {
    d: u64,
    e: address,
}

// ============================================================================
// Dynamic Field Structures
// ============================================================================

/// Test structure with mixed static and dynamic fields
#[allow(unused_field)]
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

#[allow(unused_field)]
public struct DynamicStruct_ {
    id: address,
    a: u32,
    b: bool,
    c: vector<u64>,
    d: vector<u128>,
    e: u64,
    f: u128,
    g: u256,
}

/// Test structure with various vector types
#[allow(unused_field)]
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

#[allow(unused_field)]
public struct DynamicStruct2_ {
    id: address,
    a: vector<bool>,
    b: vector<u8>,
    c: vector<u16>,
    d: vector<u32>,
    e: vector<u64>,
    f: vector<u128>,
    g: vector<u256>,
    h: vector<address>,
}

/// Test structure with nested vectors
#[allow(unused_field)]
public struct DynamicStruct3 has key {
    id: UID,
    a: vector<vector<u8>>,
    b: vector<vector<u32>>,
    c: vector<vector<u64>>,
    d: vector<vector<u128>>,
}

#[allow(unused_field)]
public struct DynamicStruct3_ {
    id: address,
    a: vector<vector<u8>>,
    b: vector<vector<u32>>,
    c: vector<vector<u64>>,
    d: vector<vector<u128>>,
}

/// Test structure with nested struct vectors
#[allow(unused_field)]
public struct DynamicStruct4 has key {
    id: UID,
    a: vector<DynamicNestedStructChild>,
    b: vector<StaticNestedStructChild>,
}

#[allow(unused_field)]
public struct DynamicStruct4_ {
    id: address,
    a: vector<DynamicNestedStructChild>,
    b: vector<StaticNestedStructChild>,
}

/// Test structure with complex nested wrappers
#[allow(unused_field)]
public struct DynamicStruct5 has key {
    id: UID,
    a: vector<NestedStructChildWrapper>,
}

#[allow(unused_field)]
public struct DynamicStruct5_ {
    id: address,
    a: vector<NestedStructChildWrapper>,
}

/// Generic test structure
#[allow(unused_field)]
public struct GenericStruct<T> has key, store {
    id: UID,
    a: vector<T>,
    b: T,
}

#[allow(unused_field)]
public struct GenericStruct_<T> {
    id: address,
    a: vector<T>,
    b: T,
}

/// Child structure for dynamic nested testing
#[allow(unused_field)]
public struct DynamicNestedStructChild has store {
    a: vector<u32>,
    b: u128,
}

/// Wrapper for nested struct children
#[allow(unused_field)]
public struct NestedStructChildWrapper has store {
    a: vector<DynamicNestedStructChild>,
    b: vector<StaticNestedStructChild>,
}

// ============================================================================
// Object Wrapping Structures
// ============================================================================

/// Simple wrapped object for testing
#[allow(unused_field)]
public struct Bar has key, store {
    id: UID,
    a: u64,
}

#[allow(unused_field)]
public struct Bar_ {
    id: address,
    a: u64,
}

/// Struct containing a wrapped object
#[allow(unused_field)]
public struct Foo has key, store {
    id: UID,
    a: u64,
    b: Bar,
    c: u32,
}

#[allow(unused_field)]
public struct Foo_ {
    id: address,
    a: u64,
    b: Bar,
    c: u32,
}

/// Struct with nested wrapped objects
#[allow(unused_field)]
public struct MegaFoo has key {
    id: UID,
    a: u64,
    b: Foo,
    c: u32,
}

#[allow(unused_field)]
public struct MegaFoo_ {
    id: address,
    a: u64,
    b: Foo,
    c: u32,
}

/// Complex struct with multiple wrapped objects and vectors
#[allow(unused_field)]
public struct Var has key {
    id: UID,
    a: Bar,
    b: Foo,
    c: vector<Bar>,
}

#[allow(unused_field)]
public struct Var_ {
    id: address,
    a: Bar,
    b: Foo,
    c: vector<Bar>,
}

#[allow(unused_field)]
public struct GenericWrapper<T> has key {
    id: UID,
    a: T,
    b: GenericStruct<T>,
    c: T
}

#[allow(unused_field)]
public struct GenericWrapper_<T> {
    id: address,
    a: T,
    b: GenericStruct<T>,
    c: T
}

// ============================================================================
// Static Field Functions
// ============================================================================

/// Save a StaticFields structure to storage
entry fun save_static_fields(
    a: u256,
    b: u128,
    c: u64,
    d: u32,
    e: u16,
    f: u8,
    g: address,
    ctx: &mut TxContext,
) {
    let struct_ = StaticFields {
        id: object::new(ctx),
        a, b, c, d, e, f, g
    };
    save_in_slot(struct_, 0);
}

/// Read a StaticFields structure from storage
entry fun read_static_fields(uid: u256): StaticFields_ {
    let struct_ = read_slot<StaticFields>(0, uid);
    let StaticFields { id, a, b, c, d, e, f, g } = struct_;
    let addr = id.to_address();
    id.delete();
    StaticFields_ {
        id: addr, a, b, c, d, e, f, g
    }
}

/// Save a StaticFields2 structure to storage
entry fun save_static_fields_2(
    a: u8,
    b: address,
    c: u64,
    d: u16,
    e: u8,
    ctx: &mut TxContext,
) {
    let struct_ = StaticFields2 {
        id: object::new(ctx),
        a, b, c, d, e
    };
    save_in_slot(struct_, 0);
}

/// Read a StaticFields2 structure from storage
entry fun read_static_fields_2(uid: u256): StaticFields2_ {
    let struct_ = read_slot<StaticFields2>(0, uid);
    let StaticFields2 { id, a, b, c, d, e } = struct_;
    let addr = id.to_address();
    id.delete();
    StaticFields2_ {
        id: addr, a, b, c, d, e
    }
}

/// Save a StaticFields3 structure to storage
entry fun save_static_fields_3(
    a: u8,
    b: address,
    c: u64,
    d: address,
    ctx: &mut TxContext,
) {
    let struct_ = StaticFields3 {
        id: object::new(ctx),
        a, b, c, d
    };
    save_in_slot(struct_, 0);
}

/// Read a StaticFields3 structure from storage
entry fun read_static_fields_3(uid: u256): StaticFields3_ {
    let struct_ = read_slot<StaticFields3>(0, uid);
    let StaticFields3 { id, a, b, c, d } = struct_;
    let addr = id.to_address();
    id.delete();
    StaticFields3_ {
        id: addr, a, b, c, d
    }
}

/// Save a StaticNestedStruct structure to storage
entry fun save_static_nested_struct(
    a: u64,
    b: bool,
    d: u64,
    e: address,
    f: u128,
    g: u32,
    ctx: &mut TxContext,
) {
    let child = StaticNestedStructChild { d, e };
    let struct_ = StaticNestedStruct {
        id: object::new(ctx),
        a, b, c: child, f, g
    };
    save_in_slot(struct_, 0);
}

/// Read a StaticNestedStruct structure from storage
entry fun read_static_nested_struct(uid: u256): StaticNestedStruct_ {
    let struct_ = read_slot<StaticNestedStruct>(0, uid);
    let StaticNestedStruct { id, a, b, c, f, g } = struct_;
    let addr = id.to_address();
    id.delete();
    StaticNestedStruct_ {
        id: addr, a, b, c, f, g
    }
}

// ============================================================================
// Dynamic Field Functions
// ============================================================================

/// Save a DynamicStruct structure to storage
entry fun save_dynamic_struct(
    a: u32,
    b: bool,
    c: vector<u64>,
    d: vector<u128>,
    e: u64,
    f: u128,
    g: u256,
    ctx: &mut TxContext,
) {
    let struct_ = DynamicStruct {
        id: object::new(ctx),
        a, b, c, d, e, f, g
    };
    save_in_slot(struct_, 0);
}

/// Read a DynamicStruct structure from storage
entry fun read_dynamic_struct(uid: u256): DynamicStruct_ {
    let struct_ = read_slot<DynamicStruct>(0, uid);
    let DynamicStruct { id, a, b, c, d, e, f, g } = struct_;
    let addr = id.to_address();
    id.delete();
    DynamicStruct_ {
        id: addr, a, b, c, d, e, f, g
    }
}

/// Save a DynamicStruct2 structure to storage
entry fun save_dynamic_struct_2(
    a: vector<bool>,
    b: vector<u8>,
    c: vector<u16>,
    d: vector<u32>,
    e: vector<u64>,
    f: vector<u128>,
    g: vector<u256>,
    h: vector<address>,
    ctx: &mut TxContext,
) {
    let struct_ = DynamicStruct2 {
        id: object::new(ctx),
        a, b, c, d, e, f, g, h
    };
    save_in_slot(struct_, 0);
}

/// Read a DynamicStruct2 structure from storage
entry fun read_dynamic_struct_2(uid: u256): DynamicStruct2_ {
    let struct_ = read_slot<DynamicStruct2>(0, uid);
    let DynamicStruct2 { id, a, b, c, d, e, f, g, h } = struct_;
    let addr = id.to_address();
    id.delete();
    DynamicStruct2_ {
        id: addr, a, b, c, d, e, f, g, h
    }
}

/// Save a DynamicStruct3 structure to storage
entry fun save_dynamic_struct_3(
    a: vector<vector<u8>>,
    b: vector<vector<u32>>,
    c: vector<vector<u64>>,
    d: vector<vector<u128>>,
    ctx: &mut TxContext,
) {
    let struct_ = DynamicStruct3 {
        id: object::new(ctx),
        a, b, c, d
    };
    save_in_slot(struct_, 0);
}

/// Read a DynamicStruct3 structure from storage
entry fun read_dynamic_struct_3(uid: u256): DynamicStruct3_ {
    let struct_ = read_slot<DynamicStruct3>(0, uid);
    let DynamicStruct3 { id, a, b, c, d } = struct_;
    let addr = id.to_address();
    id.delete();
    DynamicStruct3_ {
        id: addr, a, b, c, d
    }
}

/// Save a DynamicStruct4 structure to storage
entry fun save_dynamic_struct_4(
    x: vector<u32>,
    y: u64,
    z: u128,
    w: address,
    ctx: &mut TxContext,
) {
    let a = vector[
        DynamicNestedStructChild { a: x, b: z },
        DynamicNestedStructChild { a: x, b: z + 1 }
    ];
    let b = vector[
        StaticNestedStructChild { d: y, e: w },
        StaticNestedStructChild { d: y + 1, e: w },
        StaticNestedStructChild { d: y + 2, e: w }
    ];
    let struct_ = DynamicStruct4 {
        id: object::new(ctx),
        a, b
    };
    save_in_slot(struct_, 0);
}

/// Read a DynamicStruct4 structure from storage
entry fun read_dynamic_struct_4(uid: u256): DynamicStruct4_ {
    let struct_ = read_slot<DynamicStruct4>(0, uid);
    let DynamicStruct4 { id, a, b } = struct_;
    let addr = id.to_address();
    id.delete();
    DynamicStruct4_ {
        id: addr, a, b
    }
}

/// Save a DynamicStruct5 structure to storage
entry fun save_dynamic_struct_5(
    x: u32,
    y: u64,
    z: u128,
    w: address,
    ctx: &mut TxContext,
) {
    let v = vector[x, x + 1, x + 2];
    let a1 = vector[
        DynamicNestedStructChild { a: v, b: z },
        DynamicNestedStructChild { a: v, b: z + 1 }
    ];
    let a2 = vector[
        DynamicNestedStructChild { a: v, b: z + 2 },
        DynamicNestedStructChild { a: v, b: z + 3 },
        DynamicNestedStructChild { a: v, b: z + 4 }
    ];
    let b1 = vector[
        StaticNestedStructChild { d: y, e: w },
        StaticNestedStructChild { d: y + 1, e: w },
        StaticNestedStructChild { d: y + 2, e: w }
    ];
    let b2 = vector[
        StaticNestedStructChild { d: y + 3, e: w },
        StaticNestedStructChild { d: y + 4, e: w }
    ];
    let a = vector[
        NestedStructChildWrapper { a: a1, b: b1 },
        NestedStructChildWrapper { a: a2, b: b2 }
    ];
    let struct_ = DynamicStruct5 {
        id: object::new(ctx),
        a
    };
    save_in_slot(struct_, 0);
}

/// Read a DynamicStruct5 structure from storage
entry fun read_dynamic_struct_5(uid: u256): DynamicStruct5_ {
    let struct_ = read_slot<DynamicStruct5>(0, uid);
    let DynamicStruct5 { id, a } = struct_;
    let addr = id.to_address();
    id.delete();
    DynamicStruct5_ {
        id: addr, a
    }
}

/// Save a GenericStruct<u32> structure to storage
entry fun save_generic_struct_32(
    x: u32,
    ctx: &mut TxContext,
) {
    let a = vector[x, x + 1, x + 2];
    let struct_ = GenericStruct<u32> {
        id: object::new(ctx),
        a, b: x
    };
    save_in_slot(struct_, 0);
}

/// Read a GenericStruct<u32> structure from storage
entry fun read_generic_struct_32(uid: u256): GenericStruct_<u32> {
    let struct_ = read_slot<GenericStruct<u32>>(0, uid);
    let GenericStruct<u32> { id, a, b } = struct_;
    let addr = id.to_address();
    id.delete();
    GenericStruct_<u32> {
        id: addr, a, b
    }
}

// ============================================================================
// Object Wrapping Functions
// ============================================================================

/// Save a Foo structure to storage
entry fun save_foo(ctx: &mut TxContext) {
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

/// Read a Foo structure from storage
entry fun read_foo(uid: u256): Foo_ {
    let foo = read_slot<Foo>(0, uid);
    let Foo { id, a, b, c } = foo;
    let addr = id.to_address();
    id.delete();
    Foo_ {
        id: addr, a, b, c
    }
}

/// Save a MegaFoo structure to storage
entry fun save_mega_foo(ctx: &mut TxContext) {
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

/// Read a MegaFoo structure from storage
entry fun read_mega_foo(uid: u256): MegaFoo_ {
    let mega_foo = read_slot<MegaFoo>(0, uid);
    let MegaFoo { id, a, b, c } = mega_foo;
    let addr = id.to_address();
    id.delete();
    MegaFoo_ {
        id: addr, a, b, c
    }
}

/// Save a Var structure to storage
entry fun save_var(ctx: &mut TxContext) {
    let bar_1 = Bar {
        id: object::new(ctx),
        a: 41,
    };

    let bar_2 = Bar {
        id: object::new(ctx),
        a: 42,
    };

    let bar_3 = Bar {
        id: object::new(ctx),
        a: 43,
    };

    let bar_4 = Bar {
        id: object::new(ctx),
        a: 44,
    };

    let bar_5 = Bar {
        id: object::new(ctx),
        a: 45,
    };

    let foo = Foo {
        id: object::new(ctx),
        a: 101,
        b: bar_1,
        c: 102,
    };

    let var = Var {
        id: object::new(ctx),
        a: bar_2,
        b: foo,
        c: vector[bar_3, bar_4, bar_5],
    };

    save_in_slot(var, 0);
}

/// Read a Var structure from storage
entry fun read_var(uid: u256): Var_ {
    let var = read_slot<Var>(0, uid);
    let Var { id, a, b, c } = var;
    let addr = id.to_address();
    id.delete();
    Var_ {
        id: addr, a, b, c
    }
}

entry fun save_generic_wrapper_32(ctx: &mut TxContext) {
    let wrapper = GenericWrapper<u32> {
        id: object::new(ctx),
        a: 101,
        b: GenericStruct<u32> { id: object::new(ctx), a: vector[77, 88, 99], b: 1234 },
        c: 102,
    };
    save_in_slot(wrapper, 0);
}

entry fun read_generic_wrapper_32(uid: u256): GenericWrapper_<u32> {
    let wrapper = read_slot<GenericWrapper<u32>>(0, uid);
    let GenericWrapper<u32> { id, a, b, c } = wrapper;
    let addr = id.to_address();
    id.delete();
    GenericWrapper_<u32> {
        id: addr, a, b, c
    }
}

// Enums encoding
public enum Numbers has drop, store {
    One,
    Two,
    Three,
}

public enum Colors has drop, store {
    Red,
    Green,
    Blue,
}

// Struct with simple enums
public struct StructWithSimpleEnums has key, store {
    id: UID,
    n: Numbers,
    c: Colors,
}

public enum FooEnum has store, drop {
    A { x: u16, y: u32 },
    B(u64, u128, bool),
    C{n: Numbers, c: Colors}
}

// Variants A and C fit in the first slot, but only the first field of variant B does.
// The address ends up in the third slot because the u128 takes half of the second slot.
public struct FooAStruct has key, store {
    id: UID, // 8 bytes
    a: FooEnum, // 1 + 16 +  32 - 8 = 41 bytes
    b: address, // 20 bytes
}

public entry fun save_foo_a_struct_a(ctx: &mut TxContext) {
    let struct_ = FooAStruct {
        id: object::new(ctx),
        a: FooEnum::A{x: 42, y: 43},
        b: @0xcafecafecafecafecafecafecafecafecafecafe,
    };
    save_in_slot(struct_, 0);
}

public entry fun save_foo_a_struct_b(ctx: &mut TxContext) {
    let struct_ = FooAStruct {
        id: object::new(ctx),
        a: FooEnum::B(42, 43, true),
        b: @0xcafecafecafecafecafecafecafecafecafecafe,
    };
    save_in_slot(struct_, 0);
}

public entry fun save_foo_a_struct_c(ctx: &mut TxContext) {
    let struct_ = FooAStruct {
        id: object::new(ctx),
        a: FooEnum::C{n: Numbers::Two, c: Colors::Blue},
        b: @0xcafecafecafecafecafecafecafecafecafecafe,
    };
    save_in_slot(struct_, 0);
}

// In this case the variant B does not fit at all in the first slot.
public struct FooBStruct has key, store {
    id: UID, // 8 bytes
    a: address, // 20 bytes
    b: FooEnum, // 32 - 28 + 8 + 16 + 1 = 29
    c: u32, // 4 bytes
    d: u16, // 2 bytes
    e: bool, // 1 byte
}

public entry fun save_foo_b_struct_a(ctx: &mut TxContext) {
    let struct_ = FooBStruct {
        id: object::new(ctx),
        a: @0xcafecafecafecafecafecafecafecafecafecafe,
        b: FooEnum::A{x: 42, y: 43},
        c: 44,
        d: 45,
        e: true,
    };
    save_in_slot(struct_, 0);
}

public entry fun save_foo_b_struct_b(ctx: &mut TxContext) {
    let struct_ = FooBStruct {
        id: object::new(ctx),
        a: @0xcafecafecafecafecafecafecafecafecafecafe,
        b: FooEnum::B(42, 43, true),
        c: 44,
        d: 45,
        e: false,
    };
    save_in_slot(struct_, 0);
}

public entry fun save_foo_b_struct_c(ctx: &mut TxContext) {
    let struct_ = FooBStruct {
        id: object::new(ctx),
        a: @0xcafecafecafecafecafecafecafecafecafecafe,
        b: FooEnum::C{n: Numbers::Two, c: Colors::Blue},
        c: 44,
        d: 45,
        e: false,
    };
    save_in_slot(struct_, 0);
}

public struct BarStruct has key, store {
    id: UID,
    a: StructWithSimpleEnums,
    b: bool,
    c: u16,
    d: u32,
    e: u64,
    f: FooEnum,
    g: u128,
    h: u256,
    i: address,
}

public entry fun save_bar_struct(ctx: &mut TxContext) {
    let struct_ = BarStruct {
        id: object::new(ctx),
        a: StructWithSimpleEnums { id: object::new(ctx), n: Numbers::Two, c: Colors::Blue },
        b: true,
        c: 77,
        d: 88,
        e: 99,
        f: FooEnum::B(42, 43, true),
        g: 111,
        h: 99999999999999999,
        i: @0xcafecafecafecafecafecafecafecafecafecafe,
    };
    save_in_slot(struct_, 0);
}

