/// The module `hello_world` under named address `hello_world`.
/// The named address is set in the `Move.toml`.
module hello_world::hello_world;

const INT_AS_CONST: u32 = 100;

public fun get_constant(): u32 {
  INT_AS_CONST
}

public fun get_constant_local(): u32 {
  let x: u32 = INT_AS_CONST;
  x
}

// Forces the compiler to store literals on locals
public fun get_local(z_: u32): u32 {
  let x: u32 = 100;
  let y: u32 = 50;
  identity(x);
  identity_3(x, y);

  identity_2(x, y)
}

public fun echo(x: u32): u32 {
  identity(x)
}

public fun echo_2(x: u32, y: u32): u32 {
  identity_2(x, y)
}

fun identity(x: u32): u32 {
  x
}

fun identity_2(_x: u32, y: u32): u32 {
  y
}

fun identity_3(_x: u32, y: u32): (u32, u32) {
  (y, y)
}
