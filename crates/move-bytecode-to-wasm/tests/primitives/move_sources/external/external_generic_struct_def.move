module test::external_generic_struct_def;

public struct Foo<T: copy> has drop, copy {
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

// Dynamic abi substruct
public struct Baz<T: copy> has drop, copy {
    g: T,
    a: u16,
    b: vector<u256>,
}

public fun get_foo_1<T: copy>(g: T): Foo<T> {
    Foo {
        g,
        q: @0x1deadbeef,
        r: vector[1, 3, 0, 3, 4, 5, 6],
        s: vector[1, 5, 4, 3, 0, 3, 0],
        t: true,
        u: 41,
        v: 14242,
        w: 1424242,
        x: 142424242,
        y: 14242424242,
        z: 1424242424242,
        bar: Bar { g, a: 142, b: 14242 },
        baz: Baz { g, a: 14242, b: vector[1] },
    }
}

public fun get_foo_2<T: copy>(g: T): Foo<T> {
    Foo {
        g,
        q: @0x2deadbeef,
        r: vector[2, 3, 0, 3, 4, 5, 6],
        s: vector[2, 5, 4, 3, 0, 3, 0],
        t: true,
        u: 42,
        v: 24242,
        w: 2424242,
        x: 242424242,
        y: 24242424242,
        z: 2424242424242,
        bar: Bar { g, a: 242, b: 24242 },
        baz: Baz { g, a: 24242, b: vector[2] },
    }
}

public fun get_foo_3<T: copy>(g: T): Foo<T> {
    Foo {
        g,
        q: @0x3deadbeef,
        r: vector[3, 3, 0, 3, 4, 5, 6],
        s: vector[3, 5, 4, 3, 0, 3, 0],
        t: true,
        u: 43,
        v: 34242,
        w: 3424242,
        x: 342424242,
        y: 34242424242,
        z: 3424242424242,
        bar: Bar { g, a: 342, b: 34242 },
        baz: Baz { g, a: 34242, b: vector[3] },
    }
}
