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

public fun create_foo<T: copy>(g: T): Foo<T, u64> {
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

public fun create_foo_u32(g: u32): Foo<u32, u64> {
    create_foo(g)
}

public fun create_foo_vec_u32(g: vector<u32>): Foo<vector<u32>, u64> {
    create_foo(g)
}

public struct Fu<T: copy> has drop, copy {
    a: T,
    b: vector<T>,
}

public fun create_fu<T: copy>(t: T): Fu<T> {
    Fu {a: t, b: vector[t, t, t]}
}

public fun create_fu_u32(t: u32): Fu<u32> {
    create_fu(t)
}
