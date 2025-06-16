module 0x00::unpacking_struct;

public struct Bar has drop {
    n: u32,
    o: u128,
}

public fun echo_bar(bar: Bar): (u32, u128) {
    (bar.n, bar.o)
}
