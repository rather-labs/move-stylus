module 0x01::equality_structs;

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

public fun eq_struct_bool(a: bool, b: bool): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: b,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo == bar
}

public fun eq_struct_u8(a: u8, b: u8): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: b,
        v: 1,
        w: 2,
        x: 3,
        y: 4,
        z: 5,
    };

    foo == bar
}

public fun eq_struct_u16(a: u16, b: u16): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: b,
        w: 2,
        x: 3,
        y: 4,
        z: 5,
    };

    foo == bar
}

public fun eq_struct_u32(a: u32, b: u32): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: b,
        x: 3,
        y: 4,
        z: 5,
    };

    foo == bar
}

public fun eq_struct_u64(a: u64, b: u64): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: b,
        y: 4,
        z: 5,
    };

    foo == bar
}

public fun eq_struct_u128(a: u128, b: u128): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: b,
        z: 5,
    };

    foo == bar
}

public fun eq_struct_u256(a: u256, b: u256): bool {
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

    let bar = Foo {
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
        z: b,
    };

    foo == bar
}

public fun eq_struct_vec_stack_type(a: vector<u32>, b: vector<u32>): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: b,
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo == bar
}

public fun eq_struct_vec_heap_type(a: vector<u128>, b: vector<u128>): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: b,
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo == bar
}

public fun eq_struct_address(a: address, b: address): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: b,
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

    foo == bar
}

public fun eq_struct_struct(a: u32, b: u128, c: u32, d: u128): bool {
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

    let bar = Foo {
        p: Bar { n: c, o: d },
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

    foo == bar
}

public fun neq_struct_bool(a: bool, b: bool): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: b,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo != bar
}

public fun neq_struct_u8(a: u8, b: u8): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: b,
        v: 1,
        w: 2,
        x: 3,
        y: 4,
        z: 5,
    };

    foo != bar
}

public fun neq_struct_u16(a: u16, b: u16): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: b,
        w: 2,
        x: 3,
        y: 4,
        z: 5,
    };

    foo != bar
}

public fun neq_struct_u32(a: u32, b: u32): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: b,
        x: 3,
        y: 4,
        z: 5,
    };

    foo != bar
}

public fun neq_struct_u64(a: u64, b: u64): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: b,
        y: 4,
        z: 5,
    };

    foo != bar
}

public fun neq_struct_u128(a: u128, b: u128): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: b,
        z: 5,
    };

    foo != bar
}

public fun neq_struct_u256(a: u256, b: u256): bool {
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

    let bar = Foo {
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
        z: b,
    };

    foo != bar
}

public fun neq_struct_vec_stack_type(a: vector<u32>, b: vector<u32>): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: b,
        s: vector[1],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo != bar
}

public fun neq_struct_vec_heap_type(a: vector<u128>, b: vector<u128>): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[1],
        s: b,
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo != bar
}

public fun neq_struct_address(a: address, b: address): bool {
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

    let bar = Foo {
        p: Bar { n: 42, o: 4242 },
        q: b,
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

    foo != bar
}

public fun neq_struct_struct(a: u32, b: u128, c: u32, d: u128): bool {
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

    let bar = Foo {
        p: Bar { n: c, o: d },
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

    foo != bar
}
