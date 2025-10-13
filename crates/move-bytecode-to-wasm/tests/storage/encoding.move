/// Storage Encoding Test Module
/// 
/// This module provides comprehensive test cases for storage encoding functionality
/// in the Stylus framework. It includes various data structures and operations
/// to test different encoding scenarios including static fields, dynamic fields,
/// nested structures, and object wrapping patterns.

module test::storage_encoding;

use stylus::object::UID;
use stylus::tx_context::TxContext;
use stylus::object;

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

/// Test structure with mixed primitive types
public struct StaticFields2 has key {
    id: UID,
    a: u8,
    b: address,
    c: u64,
    d: u16,
    e: u8,
}

/// Test structure with address fields
public struct StaticFields3 has key {
    id: UID,
    a: u8,
    b: address,
    c: u64,
    d: address,
}

/// Test structure with nested static child
public struct StaticNestedStruct has key {
    id: UID,
    a: u64,
    b: bool,
    c: StaticNestedStructChild,
    f: u128,
    g: u32,
}

/// Child structure for static nested testing
public struct StaticNestedStructChild has store {
    d: u64,
    e: address,
}

// ============================================================================
// Dynamic Field Structures
// ============================================================================

/// Test structure with mixed static and dynamic fields
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

/// Test structure with various vector types
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

/// Test structure with nested vectors
public struct DynamicStruct3 has key {
    id: UID,
    a: vector<vector<u8>>,
    b: vector<vector<u32>>,
    c: vector<vector<u64>>,
    d: vector<vector<u128>>,
}

/// Test structure with nested struct vectors
public struct DynamicStruct4 has key {
    id: UID,
    a: vector<DynamicNestedStructChild>,
    b: vector<StaticNestedStructChild>,
}

/// Test structure with complex nested wrappers
public struct DynamicStruct5 has key {
    id: UID,
    a: vector<NestedStructChildWrapper>,
}

/// Generic test structure
public struct GenericStruct<T> has key, store {
    id: UID,
    a: vector<T>,
    b: T,
}

/// Child structure for dynamic nested testing
public struct DynamicNestedStructChild has store {
    a: vector<u32>,
    b: u128,
}

/// Wrapper for nested struct children
public struct NestedStructChildWrapper has store {
    a: vector<DynamicNestedStructChild>,
    b: vector<StaticNestedStructChild>,
}

// ============================================================================
// Object Wrapping Structures
// ============================================================================

/// Simple wrapped object for testing
public struct Bar has key, store {
    id: UID,
    a: u64,
}

/// Struct containing a wrapped object
public struct Foo has key, store {
    id: UID,
    a: u64,
    b: Bar,
    c: u32,
}

/// Struct with nested wrapped objects
public struct MegaFoo has key {
    id: UID,
    a: u64,
    b: Foo,
    c: u32,
}

/// Complex struct with multiple wrapped objects and vectors
public struct Var has key {
    id: UID,
    a: Bar,
    b: Foo,
    c: vector<Bar>,
}

public struct GenericWrapper<T> has key {
    id: UID,
    a: T,
    b: GenericStruct<T>,
    c: T
}

// ============================================================================
// Static Field Functions
// ============================================================================

/// Save a StaticFields structure to storage
public fun save_static_fields(
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
public fun read_static_fields(uid: u256): StaticFields {
    read_slot<StaticFields>(0, uid)
}

/// Save a StaticFields2 structure to storage
public fun save_static_fields_2(
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
public fun read_static_fields_2(uid: u256): StaticFields2 {
    read_slot<StaticFields2>(0, uid)
}

/// Save a StaticFields3 structure to storage
public fun save_static_fields_3(
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
public fun read_static_fields_3(uid: u256): StaticFields3 {
    read_slot<StaticFields3>(0, uid)
}

/// Save a StaticNestedStruct structure to storage
public fun save_static_nested_struct(
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
public fun read_static_nested_struct(uid: u256): StaticNestedStruct {
    read_slot<StaticNestedStruct>(0, uid)
}

// ============================================================================
// Dynamic Field Functions
// ============================================================================

/// Save a DynamicStruct structure to storage
public fun save_dynamic_struct(
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
public fun read_dynamic_struct(uid: u256): DynamicStruct {
    read_slot<DynamicStruct>(0, uid)
}

/// Save a DynamicStruct2 structure to storage
public fun save_dynamic_struct_2(
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
public fun read_dynamic_struct_2(uid: u256): DynamicStruct2 {
    read_slot<DynamicStruct2>(0, uid)
}

/// Save a DynamicStruct3 structure to storage
public fun save_dynamic_struct_3(
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
public fun read_dynamic_struct_3(uid: u256): DynamicStruct3 {
    read_slot<DynamicStruct3>(0, uid)
}

/// Save a DynamicStruct4 structure to storage
public fun save_dynamic_struct_4(
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
public fun read_dynamic_struct_4(uid: u256): DynamicStruct4 {
    read_slot<DynamicStruct4>(0, uid)
}

/// Save a DynamicStruct5 structure to storage
public fun save_dynamic_struct_5(
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
public fun read_dynamic_struct_5(uid: u256): DynamicStruct5 {
    read_slot<DynamicStruct5>(0, uid)
}

/// Save a GenericStruct<u32> structure to storage
public fun save_generic_struct_32(
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
public fun read_generic_struct_32(uid: u256): GenericStruct<u32> {
    read_slot<GenericStruct<u32>>(0, uid)
}

// ============================================================================
// Object Wrapping Functions
// ============================================================================

/// Save a Foo structure to storage
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

/// Read a Foo structure from storage
public fun read_foo(uid: u256): Foo {
    read_slot<Foo>(0, uid)
}

/// Save a MegaFoo structure to storage
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

/// Read a MegaFoo structure from storage
public fun read_mega_foo(uid: u256): MegaFoo {
    read_slot<MegaFoo>(0, uid)
}

/// Save a Var structure to storage
public fun save_var(ctx: &mut TxContext) {
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
public fun read_var(uid: u256): Var {
    read_slot<Var>(0, uid)
}

public fun save_generic_wrapper_32(ctx: &mut TxContext) {
    let wrapper = GenericWrapper<u32> {
        id: object::new(ctx),
        a: 101,
        b: GenericStruct<u32> { id: object::new(ctx), a: vector[77, 88, 99], b: 1234 },
        c: 102,
    };
    save_in_slot(wrapper, 0);
}

public fun read_generic_wrapper_32(uid: u256): GenericWrapper<u32> {
    read_slot<GenericWrapper<u32>>(0, uid)
}