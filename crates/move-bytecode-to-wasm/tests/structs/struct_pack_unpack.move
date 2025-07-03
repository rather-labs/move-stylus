module 0x00::struct_pack_unpack;

// Static abi struct
public struct Foo has drop {
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
public struct Bar has drop {
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

public fun echo_foo_pack(
    q: address,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
    baz: Baz
): Foo {
    Foo { q, t, u, v, w, x, y, z, baz }
}

public fun echo_bar_pack(
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
    baz: Baz
): Bar {
    Bar { q, r, s, t, u, v, w, x, y, z, bazz, baz }
}

public fun echo_foo_unpack(foo: Foo): (address, bool, u8, u16, u32, u64, u128, u256, Baz) {
    let Foo { q, t, u, v, w, x, y, z, baz } = foo;
    ( q, t, u, v, w, x, y, z, baz )
}

public fun echo_foo_unpack_ignore_fields(foo: Foo): (address, u8, u32, u128, Baz) {
    let Foo { q, t: _, u, v: _, w, x: _, y, z: _, baz } = foo;
    ( q, u, w, y, baz )
}

public fun echo_bar_unpack(bar: Bar): (address, vector<u32>, vector<u128>, bool, u8, u16, u32, u64, u128, u256, Bazz, Baz) {
    let Bar { q, r, s, t, u, v, w, x, y, z, bazz, baz } = bar;
    ( q, r, s, t, u, v, w, x, y, z, bazz, baz )
}


public fun echo_bar_unpack_ignore_fields(bar: Bar): (address, vector<u128>, u8, u32, u128, Bazz) {
    let Bar { q, r: _, s, t: _, u, v: _, w, x: _, y, z: _, bazz, baz: _ } = bar;
    ( q, s, u, w, y, bazz )
}
