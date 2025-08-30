module hello_world::hello_world;

use stylus::tx_context::TxContext;
use hello_world::other_mod::Test;

const INT_AS_CONST: u128 = 128128128;

/// Struct with generic type T
public struct Bar has drop, copy {
    a: u32,
    b: u128,
}

public struct Foo<T> has drop, copy {
    c: T,
    d: Bar,
    e: address,
    f: bool,
    g: u64,
    h: u256,
    i: vector<u32>,
}

public struct Baz<T> has drop, copy {
    c: T,
    d: Bar,
    e: address,
    f: bool,
    g: u64,
    h: u256,
}

// Enum
public enum TestEnum has drop {
    FirstVariant,
    SecondVariant,
}

/// Return a constant
public fun get_constant(): u128 {
  INT_AS_CONST
}

/// Set constant as local
public fun get_constant_local(): u128 {
  let x: u128 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_local(_z: u128): u128 {
  let x: u128 = 100;
  let y: u128 = 50;
  identity(x);

  identity_2(x, y)
}

// Forces the compiler to store literals on locals
public fun get_copied_local(): u128 {
  let x: u128 = 100;

  let y = x; // copy
  let mut _z = x; // move
  identity(y);
  identity(_z);

  _z = 111;
  y
}

public fun echo(x: u128): u128 {
  identity(x)
}

public fun echo_2(x: u128, y: u128): u128 {
  identity_2(x, y)
}

fun identity(x: u128): u128 {
  x
}

fun identity_2(_x: u128, y: u128): u128 {
  y
}

/// Exposition of EVM global variables through TxContext object
public fun tx_context_properties(ctx: &TxContext): (address, u256, u64, u256, u64, u64, u64, u256) {
    (
        ctx.sender(),
        ctx.msg_value(),
        ctx.block_number(),
        ctx.block_basefee(),
        ctx.block_gas_limit(),
        ctx.block_timestamp(),
        ctx.chain_id(),
        ctx.gas_price(),
    )
}

// Control Flow
public fun fibonacci(n: u64): u64 {
    if (n == 0) return 0;
    if (n == 1) return 1;
    let mut a = 0;
    let mut b = 1;
    let mut count = 2;
    while (count <= n) {
        let temp = a + b;
        a = b;
        b = temp;
        count = count + 1;
    };
    b
}

// Just an intrincated contrl flow example
public fun sum_special(n: u64): u64 {
    let mut total = 0;
    let mut i = 1;

    'outer: loop {
        if (i > n) {
            break
        };

        if (i > 1) {
            let mut j = 2;
            let mut x = 1;
            while (j * j <= i) {
                if (i % j == 0) {
                    x = 0;
                    break
                };
                j = j + 1;
            };

            if (x == 1) {
                total = total + 7;
            };
        };

        i = i + 1;
    };

    total
}


// Structs
public fun create_foo_u16(a: u16, b: u16): Foo<u16> {
    let mut foo = Foo {
        c: a,
        d: Bar { a: 42, b: 4242 },
        e: @0x7357,
        f: true,
        g: 1,
        h: 2,
        i: vector[0xFFFFFFFF],
    };

    foo.c = b;

    foo
}

public fun create_foo2_u16(a: u16, b: u16): (Foo<u16>, Foo<u16>) {
    let mut foo = Foo {
        c: a,
        d: Bar { a: 42, b: 4242 },
        e: @0x7357,
        f: true,
        g: 1,
        h: 2,
        i: vector[0xFFFFFFFF],
    };

    foo.c = b;

    (foo, copy(foo))
}

public fun create_baz_u16(a: u16, _b: u16): Baz<u16> {
    let baz = Baz {
        c: a,
        d: Bar { a: 42, b: 4242 },
        e: @0x7357,
        f: true,
        g: 1,
        h: 2,
    };

    baz
}

public fun create_baz2_u16(a: u16, _b: u16): (Baz<u16>, Baz<u16>) {
    let baz = Baz {
        c: a,
        d: Bar { a: 42, b: 4242 },
        e: @0x7357,
        f: true,
        g: 1,
        h: 2,
    };

    (baz, copy(baz))
}

public fun multi_values_1(): (vector<u32>, vector<u128>, bool, u64) {
    (vector[0xFFFFFFFF, 0xFFFFFFFF], vector[0xFFFFFFFFFF_u128], true, 42)
}

public fun multi_values_2(): (u8, bool, u64) {
    (84, true, 42)
}

// Enums
public fun echo_variant(x:  TestEnum): TestEnum {
    x
}

// Use structs from other modules defined by us
public fun test_values(test: &Test): (u8, u8) {
    test.get_test_values()
}
