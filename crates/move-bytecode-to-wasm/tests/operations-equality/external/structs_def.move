module test::equality_external_structs_def;

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

public fun create_foo_u8(x: u8): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: x,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    }
}

public fun create_foo_u16(x: u16): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: x,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    }
}

public fun create_foo_u32(x: u32): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: x,
        x: 4,
        y: 5,
        z: 6,
    }
}

public fun create_foo_u64(x: u64): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: x,
        y: 5,
        z: 6,
    }
}

public fun create_foo_u128(x: u128): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: x,
        z: 6,
    }
}

public fun create_foo_u256(x: u256): Foo {
    Foo {
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
        z: x,
    }
}

public fun create_foo_bool(x: bool): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: x,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    }
}

public fun create_foo_address(x: address): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: x,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    }
}

public fun create_foo_struct(n: u32, o: u128): Foo {
    Foo {
        p: Bar { n, o },
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
    }
}

public fun create_foo_vec_stack_type(r: vector<u32>): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r,
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    }
}

public fun create_foo_vec_heap_type(s: vector<u128>): Foo {
    Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s,
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    }
}
