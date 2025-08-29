module 0x00::generic_functions;

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

public fun echo_struct(x: Foo): Foo {
    generic_1(x)
}

public fun echo_struct_ref(inner: bool, x: &Foo): &Foo {
    generic_4(inner, x)
}

public fun echo_struct_mut_ref(inner: bool, x: &mut Foo): &mut Foo {
    generic_5(inner, x)
}

public fun echo_u8(x: u8): u8 {
    generic_1(x)
}

public fun echo_u16(x: u16): u16 {
    generic_1(x)
}

public fun echo_u32(x: u32): u32 {
    generic_1(x)
}

public fun echo_u64(x: u64): u64 {
    generic_1(x)
}

public fun echo_u128(x: u128): u128 {
    generic_1(x)
}

public fun echo_u256(x: u256): u256 {
    generic_1(x)
}

public fun echo_address(x: address): address {
    generic_1(x)
}

public fun echo_vec_u32(x: vector<u32>): vector<u32> {
    generic_1(x)
}

public fun echo_vec_u128(x: vector<u128>): vector<u128> {
    generic_1(x)
}

public fun echo_vec_u128_ref(x: &vector<u128>): &vector<u128> {
    generic_4(false, x)
}

public fun echo_vec_u128_mut_ref(x: &mut vector<u128>): &mut vector<u128> {
    generic_5(false, x)
}

public fun echo_u32_u128(x: u32, y: u128): (u32, u128) {
    generic_3(x, y)
}

public fun echo_address_vec_u128(x: address, y: vector<u128>): (address, vector<u128>) {
    generic_3(x, y)
}

public fun echo_struct_vec_u128(x: Foo, y: vector<u128>): (Foo, vector<u128>) {
    generic_3(x, y)
}

fun generic_1<T>(t: T): T {
    generic_2(t)
}

fun generic_2<T>(t: T): T {
    t
}

fun generic_3<T, U>(t: T, u: U): (T, U) {
    (generic_2(t), generic_2(u))
}

fun generic_4<T>(inner: bool, t: &T): &T {
    if (inner) {
        inner_generic_4(t)
    } else {
        t
    }
}

fun inner_generic_4<T>(t: &T): &T {
    t
}

fun generic_5<T>(inner: bool, t: &mut T): &mut T {
    if (inner) {
        inner_generic_5(t)
    } else {
        t
    }
}

fun inner_generic_5<T>(t: &mut T): &mut T {
    t
}