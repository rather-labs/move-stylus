module hello_world::hello_world;

use stylus::tx_context::TxContext;
use hello_world::other_mod::Test;
use hello_world::another_mod::AnotherTest;

const INT_AS_CONST: u128 = 128128128;

/// Struct with generic type T
public struct Bar has drop {
    n: u32,
    o: u128,
}

public struct Foo<T> has drop {
    g: T,
    p: Bar,
    q: address,
    r: vector<u32>,
    // s: vector<u128>,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
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
  let mut z = x; // move
  identity(y);
  identity(z);

  z = 111;
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

// Inteaction with signer
public fun echo_signer_with_int(x: signer, y: u8): (u8, signer) {
    (y, x)
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

public fun sum_special(n: u64): u64 {
    let mut total = 0;
    let mut i = 1;

    'outer: loop {
        if (i > n) {
            break // Exit main loop
        };

        // Check if i is prime using a while loop
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
        g: a,
        p: Bar { n: 42, o: 4242 },
        q: @0x7357,
        r: vector[0xFFFFFFFF],
       // s: vector[6],
        t: true,
        u: 1,
        v: 2,
        w: 3,
        x: 4,
        y: 5,
        z: 6,
    };

    foo.g = a;
    foo.v = b;

    foo
}

// Enums
public fun echo_variant(x:  TestEnum): TestEnum {
    x
}

// Use structs from other modules defined by us
public fun test_values(test: &Test): (u8, u8) {
    test.get_test_values()
}
