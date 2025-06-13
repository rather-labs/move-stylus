module 0x00::struct_mut_fields;

public struct Bar has drop {
    n: u32,
    o: u128,
}

public struct Foo has drop {
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

public fun echo_mut_bool(a: bool): bool {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.t = a;
    foo.t
}

public fun echo_mut_u8(a: u8): u8 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.u = a;
    foo.u
}

public fun echo_mut_u16(a: u16): u16 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.v = a;
    foo.v
}

public fun echo_mut_u32(a: u32): u32 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.w = a;
    foo.w
}

public fun echo_mut_u64(a: u64): u64 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.x = a;
    foo.x
}

public fun echo_mut_u128(a: u128): u128 {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.y = a;
    foo.y
}

public fun echo_mut_u256(a: u256): u256{
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.z = a;
    foo.z
}

public fun echo_mut_vec_stack_type(a: vector<u32>): vector<u32> {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.r = a;
    foo.r
}

public fun echo_mut_vec_heap_type(a: vector<u128>): vector<u128> {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.s = a;
    foo.s
}

public fun echo_mut_address(a: address): address {
    let mut foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: false,
        u: 2,
        v: 3,
        w: 4,
        x: 5,
        y: 6,
        z: 7,
    };

    foo.q = a;
    foo.q
}

public fun echo_bar_struct_fields(a: u32, b: u128): (u32, u128) {
    let mut foo = Foo {
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

    foo.p.n = a;
    foo.p.o = b;

    (foo.p.n, foo.p.o)
}
