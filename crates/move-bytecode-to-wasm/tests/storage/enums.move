module test::enums;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

// ============================================================================
// STRUCT DEFINITIONS
// ============================================================================

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

entry fun test_foo_struct(s: &mut FooStruct) {
    set_variant(s, FooEnum::A { x: 5, y: 6 });
    match (&s.a) {
        FooEnum::A { x, y } => {
            assert!(x == 5, 5);
            assert!(y == 6, 6);
        },
        _ => abort(1),
    };

    set_variant(s, FooEnum::B(3, 4, true));
    match (&s.a) {
        FooEnum::B(x, y, z) => {
            assert!(x == 3, 1);
            assert!(y == 4, 2);
            assert!(z == true, 3);
        },
        _ => abort(1),
    };

    set_variant(s, FooEnum::C{n: Numbers::Two, c: Colors::Green});
    match (&s.a) {
        FooEnum::C{n, c} => {
            assert!(n == Numbers::Two, 3);
            assert!(c == Colors::Green, 4);
        },
        _ => abort(1),
    };
}

entry fun destroy_foo_struct(s: FooStruct) {
    let FooStruct { id, a: _ } = s;
    object::delete(id);
}

