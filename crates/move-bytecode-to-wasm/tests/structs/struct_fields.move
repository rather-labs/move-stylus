module 0x00::struct_fields;

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

public fun echo_bool(a: bool): bool {
    let foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: a,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.t
}

public fun echo_u8(a: u8): u8 {
    let foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: a,
        v: 1,
        w: 2,
        x: 3,
        y: 4,
        z: 5,
    };

    foo.u
}

public fun echo_u16(a: u16): u16 {
    let foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: a,
        w: 2,
        x: 3,
        y: 4,
        z: 5,
    };

    foo.v
}

public fun echo_u32(a: u32): u32 {
    let foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: a,
        x: 3,
        y: 4,
        z: 5,
    };

    foo.w
}

public fun echo_u64(a: u64): u64 {
    let foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: a,
        y: 4,
        z: 5,
    };

    foo.x
}

public fun echo_u128(a: u128): u128 {
    let foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: a,
        z: 5,
    };

    foo.y
}

public fun echo_u256(a: u256): u256{
    let foo = Foo {
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
        z: a,
    };

    foo.z
}

public fun echo_vec_stack_type(a: vector<u32>): vector<u32> {
    let foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: a,
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.r
}

public fun echo_vec_heap_type(a: vector<u128>): vector<u128> {
    let foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: a,
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.s
}

public fun echo_address(a: address): address {
    let foo = Foo {
        p: Bar { n: 42, o: 4242 },
        q: a,
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

    foo.q
}

public fun echo_bar_struct_fields(a: u32, b: u128): (u32, u128) {
    let foo = Foo {
        p: Bar { n: a, o: b },
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

    (foo.p.n, foo.p.o)
}
