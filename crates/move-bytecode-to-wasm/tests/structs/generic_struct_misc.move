module 0x00::generic_struct_misc;

public struct Foo<T: copy, phantom U> has drop, copy {
    g: T,
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
    bar: Bar<T>,
    baz: Baz<T>,
}

// Static abi sub-struct
public struct Bar<T: copy> has drop, copy {
    g: T,
    a: u16,
    b: u128,
}

// Dynamic abi sub-struct
public struct Baz<T: copy> has drop, copy {
    g: T,
    a: u16,
    b: vector<u256>,
}

fun create_foo<T: copy>(g: T): Foo<T, u64> {
    Foo {
        g,
        q: @0xcafe000000000000000000000000000000007357,
        r: vector[0, 3, 0, 3, 4, 5, 6],
        s: vector[6, 5, 4, 3, 0, 3, 0],
        t: true,
        u: 42,
        v: 4242,
        w: 424242,
        x: 42424242,
        y: 4242424242,
        z: 424242424242,
        bar: Bar { g, a: 42, b: 4242 },
        baz: Baz { g, a: 4242, b: vector[3] },
    }
}

entry fun create_foo_u32(g: u32): Foo<u32, u64> {
    create_foo(g)
}

entry fun create_foo_vec_u32(g: vector<u32>): Foo<vector<u32>, u64> {
    create_foo(g)
}

public struct Fu<T: copy> has drop, copy {
    a: T,
    b: vector<T>,
}

fun create_fu<T: copy>(t: T): Fu<T> {
    Fu {a: t, b: vector[t, t, t]}
}

entry fun create_fu_u32(t: u32): Fu<u32> {
    create_fu(t)
}

public struct GenericStruct<S, T, U> has drop, copy {
    a: S,
    b: T,
    c: U,
}

fun inner_create_generic_struct<T>(a: u16, b: T, c: u64): GenericStruct<u16, T, u64> {
    GenericStruct {a, b, c}
}

entry fun create_generic_struct(a: u16, b: u32, c: u64): GenericStruct<u16, u32, u64> {
    inner_create_generic_struct<u32>(a, b, c)
}

public struct ComplexGenericStruct<S, T, U> has drop, copy {
    a: S,
    b: GenericStruct<S, T, U>,
    c: vector<U>,
    d: vector<vector<U>>,
}

fun inner_create_complex_generic_struct<T, U: copy>(a: u16, b: T, c: U): ComplexGenericStruct<u16, T, U> {
    ComplexGenericStruct {a, b: GenericStruct {a, b, c}, c: vector[c, c, c], d: vector[vector[c], vector[c, c]]}
}

fun inner_pack_unpack_generic_struct<T: drop, U: drop + copy>(a: u16, t: T, u: U) {
    let s = ComplexGenericStruct {a, b: GenericStruct{a, b: t, c: u}, c: vector[u, u, u], d: vector[vector[u], vector[u, u]]};
    let ComplexGenericStruct {a: _a_val, b: _b, c: _c, d: _d} = s;
}

entry fun create_complex_generic_struct(a: u16, b: u32, c: u64): ComplexGenericStruct<u16, u32, u64> {
    inner_pack_unpack_generic_struct(a, b, c);
    inner_create_complex_generic_struct<u32, u64>(a, b, c)
}