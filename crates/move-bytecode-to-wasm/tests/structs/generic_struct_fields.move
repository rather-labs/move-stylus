module 0x00::generic_struct_fields;

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

entry fun echo_bool(a: bool): (bool, bool) {
    let foo = Foo {
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

    (foo.g, foo.t)
}

entry fun echo_u8(a: u8): (u8, u8) {
    let foo = Foo {
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

    (foo.g, foo.u)
}

entry fun echo_u16(a: u16): (u16, u16) {
    let foo = Foo {
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

    (foo.g, foo.v)
}

entry fun echo_u32(a: u32): (u32, u32) {
    let foo = Foo {
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

    (foo.g, foo.w)
}

entry fun echo_u64(a: u64): (u64, u64) {
    let foo = Foo {
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

    (foo.g, foo.x)
}

entry fun echo_u128(a: u128): (u128, u128) {
    let foo = Foo {
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

    (foo.g, foo.y)
}

entry fun echo_u256(a: u256): (u256, u256) {
    let foo = Foo {
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

    (foo.g, foo.z)
}

entry fun echo_vec_stack_type(a: vector<u32>): (vector<u32>, vector<u32>) {
    let foo = Foo {
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

    (foo.g, foo.r)
}

entry fun echo_vec_heap_type(a: vector<u128>): (vector<u128>, vector<u128>) {
    let foo = Foo {
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

    (foo.g, foo.s)
}

entry fun echo_address(a: address): (address, address) {
    let foo = Foo {
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

    (foo.g, foo.q)
}

entry fun echo_bar_struct_fields(a: u32, b: u128): (u32, u128, u32, u128) {
    let foo = Foo {
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

    (foo.g.n, foo.g.o, foo.p.n, foo.p.o)
}
