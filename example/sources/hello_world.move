module 0x01::hello_world;

public fun echo_with_int(x: signer, y: u8): (u8, signer) {
    (y, x)
}
