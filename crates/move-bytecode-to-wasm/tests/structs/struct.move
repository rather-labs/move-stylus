public struct Foo{ x: u64, y: bool }

public fun echo_bool(a: bool, b: u64): bool {
    let foo = Foo { x: b, y: a }:
    foo.a
}
