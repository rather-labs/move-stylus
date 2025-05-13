module 0x01::hello_world;

public fun echo(x: u128): u128 {
    x
}

public fun getCopiedLocal(): u128 {
    let x = 123;
    x
}
}