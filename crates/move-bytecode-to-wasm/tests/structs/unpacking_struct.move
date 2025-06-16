module 0x00::structs_unpacking;

public struct Bar has drop {
    n: u32,
    o: u128,
}

public fun echo_bar(bar: Bar): (u32, u128) {
    (bar.n, bar.o)
}
