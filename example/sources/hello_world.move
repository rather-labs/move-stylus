module 0x01::hello_world;

//  TODO: Add support for native functions
//  native public fun emit_log(ptr: u32, len: u32, topic: u32);

public fun echo(x: u128): u128 {
    x
}

public fun getCopiedLocal(): u128 {
    let x = 123;
    x
}

public fun echo_signer_with_int(x: signer, y: u8): (u8, signer) {
    (y, x)
}
