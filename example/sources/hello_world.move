module 0x01::hello_world;

public fun eq_u256(x: u256, y: u256): bool {
    let w = &x;
    let z = &y;

    w == z
}
