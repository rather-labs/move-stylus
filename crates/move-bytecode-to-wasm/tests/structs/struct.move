module 0x00::structs;

public struct Foo has drop {
    x: u64,
    y: bool
}

public fun echo_u64(a: u64): u64 {
    let foo = Foo { x: a, y: true };
    foo.x
}

public fun echo_bool(a: bool): bool {
    let foo = Foo { x: 42, y: a };
    foo.y
}

