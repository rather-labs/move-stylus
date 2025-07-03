module 0x00::generic_struct_unpack;

// Static abi struct
public struct Foo<T> has drop {
    g: T,
    q: address,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
    baz: Baz,
}

// Dynamic abi struct
public struct Bar<T> has drop {
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
    bazz: Bazz,
    baz: Baz,
}

// Static abi sub-struct
#[allow(unused_field)]
public struct Baz has drop {
    a: u16,
    b: u128,
}

// Dynamic abi substruct
#[allow(unused_field)]
public struct Bazz has drop {
    a: u16,
    b: vector<u256>,
}

public fun echo_foo_unpack(foo: Foo<u32>): (u32, address, bool, u8, u16, u32, u64, u128, u256, Baz) {
    let Foo { g, q, t, u, v, w, x, y, z, baz } = foo;
    ( g, q, t, u, v, w, x, y, z, baz )
}

public fun echo_bar_unpack(bar: Bar<vector<u32>>): (vector<u32>, address, vector<u32>, vector<u128>, bool, u8, u16, u32, u64, u128, u256, Bazz, Baz) {
    let Bar { g, q, r, s, t, u, v, w, x, y, z, bazz, baz } = bar;
    ( g, q, r, s, t, u, v, w, x, y, z, bazz, baz )
}
