module 0x00::struct_abi_packing_unpacking;

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
public struct Baz has drop {
    a: u16,
    b: u128,
}

// Dynamic abi substruct
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
    ba: u16,
    bb: u128
): Foo {
    Foo { q, t, u, v, w, x, y, z, baz: Baz { a: ba, b: bb } }
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
    ba: u16,
    bb: vector<u256>,
    bba: u16,
    bbb: u128,
): Bar {
    Bar { q, r, s, t, u, v, w, x, y, z, bazz: Bazz { a: ba, b: bb }, baz: Baz { a: bba, b: bbb } }
}

public fun echo_foo_unpack(foo: Foo): (address, bool, u8, u16, u32, u64, u128, u256, u16, u128) {
    (
        foo.q,
        foo.t,
        foo.u,
        foo.v,
        foo.w,
        foo.x,
        foo.y,
        foo.z,
        foo.baz.a,
        foo.baz.b,
    )
}

public fun echo_bar_unpack(bar: Bar): (address, vector<u32>, vector<u128>, bool, u8, u16, u32, u64, u128, u256, u16, vector<u256>, u16, u128) {
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
        bar.baz.a,
        bar.baz.b,
    )
}


public fun pack_unpack_static(foo: Foo): Foo {
    Foo {
        q: foo.q,
        t: foo.t,
        u: foo.u,
        v: foo.v,
        w: foo.w,
        x: foo.x,
        y: foo.y,
        z: foo.z,
        baz: Baz { a: foo.baz.a, b: foo.baz.b },
    }
}

public fun pack_unpack_dynamic(bar: Bar): Bar {
    Bar {
        q: bar.q,
        r: bar.r,
        s: bar.s,
        t: bar.t,
        u: bar.u,
        v: bar.v,
        w: bar.w,
        x: bar.x,
        y: bar.y,
        z: bar.z,
        bazz: Bazz { a: bar.bazz.a, b: bar.bazz.b },
        baz: Baz { a: bar.baz.a, b: bar.baz.b },
    }
}

// This tests the packing/unpacking with the struct between other values
public fun pack_unpack_between_vals_static(v1: bool, foo: Foo, v4: vector<u128>): (bool, Foo, vector<u128>) {
    (
        v1,
        Foo {
            q: foo.q,
            t: foo.t,
            u: foo.u,
            v: foo.v,
            w: foo.w,
            x: foo.x,
            y: foo.y,
            z: foo.z,
            baz: Baz { a: foo.baz.a, b: foo.baz.b },
        },
        v4
    )
}

public fun pack_unpack_between_vals_dynamic(v1: bool, _v2: vector<u32>, bar: Bar, _v3: bool, v4: vector<u128>): (bool, Bar, vector<u128>) {
    (
        v1,
        Bar {
            q: bar.q,
            r: bar.r,
            s: bar.s,
            t: bar.t,
            u: bar.u,
            v: bar.v,
            w: bar.w,
            x: bar.x,
            y: bar.y,
            z: bar.z,
            bazz: Bazz { a: bar.bazz.a, b: bar.bazz.b },
            baz: Baz { a: bar.baz.a, b: bar.baz.b },
        },
        v4
    )
}
