module 0x00::generic_struct_mut_fields;

public struct Bar has drop {
    n: u32,
    o: u128,
}

public struct Foo<T> has drop {
    g: T,
    p: Bar,
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
}

public fun echo_bool(a: bool, b: bool): (bool, bool) {
    let mut foo = Foo {
        g: true,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.t = b;

    (foo.g, foo.t)
}

public fun echo_u8(a: u8, b: u8): (u8, u8) {
    let mut foo = Foo {
        g: 1,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.u = b;

    (foo.g, foo.u)
}

public fun echo_u16(a: u16, b: u16): (u16, u16) {
    let mut foo = Foo {
        g: a,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.v = b;

    (foo.g, foo.v)
}

public fun echo_u32(a: u32, b: u32): (u32, u32) {
    let mut foo = Foo {
        g: a,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.w = b;

    (foo.g, foo.w)
}

public fun echo_u64(a: u64, b: u64): (u64, u64) {
    let mut foo = Foo {
        g: a,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.x = b;

    (foo.g, foo.x)
}

public fun echo_u128(a: u128, b: u128): (u128, u128) {
    let mut foo = Foo {
        g: a,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.y = b;

    (foo.g, foo.y)
}

public fun echo_u256(a: u256, b: u256): (u256, u256) {
    let mut foo = Foo {
        g: a,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.z = b;

    (foo.g, foo.z)
}

public fun echo_vec_stack_type(a: vector<u32>, b: vector<u32>): (vector<u32>, vector<u32>) {
    let mut foo = Foo {
        g: a,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.r = b;

    (foo.g, foo.r)
}

public fun echo_vec_heap_type(a: vector<u128>, b: vector<u128>): (vector<u128>, vector<u128>) {
    let mut foo = Foo {
        g: a,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.s = b;

    (foo.g, foo.s)
}

public fun echo_address(a: address, b: address): (address, address) {
    let mut foo = Foo {
        g: a,
        p: Bar { n: 42, o: 4242 },
        q: @0xdeadbeef,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.q = b;

    (foo.g, foo.q)
}

public fun echo_bar_struct_fields(a: u32, b: u128, c: u32, d: u128): (u32, u128, u32, u128) {
    let mut foo = Foo {
        g: Bar { n: a, o: b },
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g.n = a;
    foo.g.o = b;
    foo.p.n = c;
    foo.p.o = d;

    (foo.g.n, foo.g.o, foo.p.n, foo.p.o)
}
