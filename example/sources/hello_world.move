module 0x01::hello_world;

//  TODO: Add support for native functions
//  native public fun emit_log(ptr: u32, len: u32, topic: u32);

public fun cast(x: u128): u8 {
    x as u8
}
