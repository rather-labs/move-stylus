/// The module `hello_world` under named address `hello_world`.
/// The named address is set in the `Move.toml`.
module hello_world::hello_world;

const ITEM_PRICE: u32 = 100;

/// Returns the "Hello, World!" as a `String`.
public fun hello_world(): u32 {
  ITEM_PRICE
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
