module 0x00::struct_packing;

public struct Baz has drop {
    a: u16,
    b: u128,
}

// static abi struct
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

public fun echo_foo(
    q: address,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
    ba: u16,
    bb: u128
): Foo {
    Foo { q, t, u, v, w, x, y, z, baz: Baz { a: ba, b: bb } }
}

/*
public struct Bazz has drop {
    a: u16,
    b: vector<u256>,
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
}

public fun echo_bar(bar: Bar): (address, vector<u32>, vector<u128>, bool, u8, u16, u32, u64, u128, u256, u16, vector<u256>) {
    (
        bar.q,
        bar.r,
        bar.s,
        bar.t,
        bar.u,
        bar.v,
        bar.w,
        bar.x,
        bar.y,
        bar.z,
        bar.bazz.a,
        bar.bazz.b,
    )
}
*/
