module 0x00::reference_args;

#[allow(unused_field)]
public struct Bar has drop {
    a: u32,
    b: u128,
}

#[allow(unused_field)]
public struct Foo has drop {
    c: Bar,
    d: address,
    e: vector<u128>,
    f: bool,
    g: u16,
    h: u256,
}

public fun test_forward(x: &u32, b: bool): (bool, &u32) {
    if (b) {
        test(x, b)
    } else {
        test_inv(b, x)
    }
}

public fun test(x: &u32, b: bool): (bool, &u32) {
    (b, x)
}

public fun test_inv(b: bool, x: &u32): (bool, &u32) {
    (b, x)
}

public fun test_mix(x: &u32, b: bool, v: u64, w: &u64): (bool, &u32, u64, &u64) {
    (b, x, v, w)
}

public fun test_forward_generics(x: &u32, b: bool, y: &mut u64): (bool, &mut u64, &u32) {
    if (b) {
        test_generics(x, b, y)
    } else {
       (b, y, x)
    }
}

public fun test_forward_generics_2(bar: &Bar, x: u128, foo: &mut Foo): (u128, &mut Foo, &Bar) {
    test_generics(bar, x, foo)
}

public fun test_generics<T, U, V>(x: &T, b: U, y: &mut V): (U, &mut V, &T) {
    (b, y, x)
}
