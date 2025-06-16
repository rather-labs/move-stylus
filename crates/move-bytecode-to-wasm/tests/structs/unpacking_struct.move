module 0x00::unpacking_struct;

public struct Foo has drop {
    q: address,
    // r: vector<u32>,
    // s: vector<u128>,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
}

// TODO: Add another struct when packing is done
// public fun echo_foo(foo: Foo): (address, vector<u32>, vector<u128>, bool, u8, u16, u32, u64, u128, u256) {
public fun echo_foo(foo: Foo): (address, bool, u8, u16, u32, u64, u128, u256) {
    (
        foo.q,
        // foo.r,
        // foo.s,
        foo.t,
        foo.u,
        foo.v,
        foo.w,
        foo.x,
        foo.y,
        foo.z,
    )
}
