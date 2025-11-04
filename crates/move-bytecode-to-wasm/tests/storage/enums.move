module test::enums;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

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

entry fun create_struct_with_simple_enums(recipient: address, ctx: &mut TxContext) {
    let s = StructWithSimpleEnums {
        id: object::new(ctx),
        n: Numbers::One,
        c: Colors::Red,
    };
    transfer::transfer(s, recipient);
}

entry fun get_struct_with_simple_enums(s: &StructWithSimpleEnums): &StructWithSimpleEnums {
    s
}

entry fun set_number(s: &mut StructWithSimpleEnums, n: Numbers) {
    s.n = n;
}

entry fun set_color(s: &mut StructWithSimpleEnums, c: Colors) {
    s.c = c;
}

entry fun get_number(s: &StructWithSimpleEnums): &Numbers {
    &s.n
}

entry fun get_color(s: &StructWithSimpleEnums): &Colors {
    &s.c
}

entry fun destroy_struct_with_simple_enums(s: StructWithSimpleEnums) {
    let StructWithSimpleEnums { id, n, c } = s;
    object::delete(id);
}

public enum FooEnum has store, drop {
    A { x: u16, y: u32 },
    B(u64, u128, bool),
    C{n: Numbers, c: Colors}
}

// Struct with not-simple enum
public struct FooStruct has key, store {
    id: UID,
    a: FooEnum,
}

entry fun create_foo_struct(recipient: address, ctx: &mut TxContext) {
    let s = FooStruct {
        id: object::new(ctx),
        a: FooEnum::A { x: 1, y: 2 },
    };
    transfer::transfer(s, recipient);
}

fun set_variant(s: &mut FooStruct, a: FooEnum) {
    s.a = a;
}

entry fun set_variant_a(s: &mut FooStruct, x: u16, y: u32) {
    s.a = FooEnum::A { x, y };
}

entry fun set_variant_b(s: &mut FooStruct, x: u64, y: u128, z: bool) {
    s.a = FooEnum::B(x, y, z);
}

entry fun set_variant_c(s: &mut FooStruct, n: Numbers, c: Colors) {
    s.a = FooEnum::C{n, c};
}

entry fun get_variant_a(s: &FooStruct): (&u16, &u32) {
    match (&s.a) {
        FooEnum::A { x, y } => (x, y),
        _ => abort(1),
    }
}

entry fun get_variant_b(s: &FooStruct): (&u64, &u128, &bool) {
    match (&s.a) {
        FooEnum::B(x, y, z) => (x, y, z),
        _ => abort(1),
    }
}

entry fun get_variant_c(s: &FooStruct): (&Numbers, &Colors) {
    match (&s.a) {
        FooEnum::C{n, c} => (n, c),
        _ => abort(1),
    }
}

entry fun destroy_foo_struct(s: FooStruct) {
    let FooStruct { id, a: _ } = s;
    object::delete(id);
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

entry fun create_bar_struct(recipient: address, ctx: &mut TxContext) {
    let s = BarStruct {
        id: object::new(ctx),
        a: StructWithSimpleEnums {
            id: object::new(ctx),
            n: Numbers::Two,
            c: Colors::Blue,
        },
        b: true,
        c: 77,
        d: 88,
        e: 99,
        f: FooEnum::B(42, 43, true),
        g: 111,
        h: 99999999999999999,
        i: @0xffffffffffffffffffffffffffffffffffffffff,
    };
    transfer::transfer(s, recipient);
}

entry fun get_foo_enum_variant_a(s: &BarStruct): (&u16, &u32) {
    match (&s.f) {
        FooEnum::A { x, y } => {
            (x, y)
        },
        _ => abort(1),
    }
}

entry fun get_foo_enum_variant_b(s: &BarStruct): (&u64, &u128, &bool) {
    match (&s.f) {
        FooEnum::B(x, y, z) => {
            (x, y, z)
        },
        _ => abort(1),
    }
}

entry fun get_foo_enum_variant_c(s: &BarStruct): (&Numbers, &Colors) {
    match (&s.f) {
        FooEnum::C{n, c} => {
            (n, c)
        },
        _ => abort(1),
    }
}

entry fun set_foo_enum_variant_a(s: &mut BarStruct, x: u16, y: u32) {
    s.f = FooEnum::A { x, y };
}

entry fun set_foo_enum_variant_b(s: &mut BarStruct, x: u64, y: u128, z: bool) {
    s.f = FooEnum::B(x, y, z);
}

entry fun set_foo_enum_variant_c(s: &mut BarStruct, n: Numbers, c: Colors) {
    s.f = FooEnum::C{n, c};
}

entry fun get_address(s: &BarStruct): &address {
    &s.i
}

entry fun destroy_bar_struct(s: BarStruct) {
    let BarStruct { id: bar_id, a, b: _, c: _, d: _, e: _, f: _, g: _, h: _, i: _ } = s;
    let StructWithSimpleEnums { id: simple_id, n: _, c: _ } = a;
    object::delete(simple_id);
    object::delete(bar_id);
}