/// The module `hello_world` under named address `hello_world`.
/// The named address is set in the `Move.toml`.
module hello_world::hello_world;

const ITEM_PRICE: u64 = 100;

/// Returns the "Hello, World!" as a `String`.
public fun hello_world(): u64 {
    ITEM_PRICE
}
