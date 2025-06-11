module 0x00::structs;

public struct Foo has drop {
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
}

public fun echo_u64(a: u64): u64 {
    let foo = Foo {
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

public fun echo_bool(a: bool): bool {
    let foo = Foo {
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

