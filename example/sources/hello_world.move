module 0x01::hello_world;

//  TODO: Add support for native functions
//  native public fun emit_log(ptr: u32, len: u32, topic: u32);

public fun cast_u8(x: u16): u8 {
    x as u8
}

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

public fun sum8(x: u8, y: u8): u8 {
    x + y
}

public fun sum16(x: u16, y: u16): u16 {
    x + y
}

public fun sum32(x: u32, y: u32): u32 {
    x + y
}

public fun sum64(x: u64, y: u64): u64 {
    x + y
}
