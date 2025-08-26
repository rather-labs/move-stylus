module test::external_struct_def;

public struct Foo has drop, copy {
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
    bar: Bar,
    baz: Baz,
}

// Static abi sub-struct
public struct Bar has drop, copy {
    a: u16,
    b: u128,
}

// Dynamic abi substruct
public struct Baz has drop, copy {
    a: u16,
    b: vector<u256>,
}

public fun get_foo(): Foo {
    Foo {
        q: @0xdeadbeef,
        r : vector[0, 3, 0, 3, 4, 5, 6],
        s : vector[6, 5, 4, 3, 0, 3, 0],
        t : false,
        u : 42,
        v : 4242,
        w : 424242,
        x : 42424242,
        y : 4242424242,
        z : 424242424242,
        bar: Bar { a: 42, b: 4242 },
        baz: Baz { a: 4242, b: vector[3] },
    }
}
