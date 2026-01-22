# Ability: drop

The `drop` ability allows instances of a struct to be *discarded* without being used. This means that when a value of a struct type with the `drop` ability goes out of scope, it can be safely ignored and removed from memory without any special handling. This is a safety feature in Move language that ensures that all assets are properly managed. Ignoring a value with the `drop` ability results in a compilation error, preventing accidental loss of resources.

```move
module book::drop_ability;

/// This struct has the `drop` ability.
public struct IgnoreMe has drop {
    a: u8,
    b: u8,
}

/// This struct does not have the `drop` ability.
public struct NoDrop {}

#[test]
// Create an instance of the `IgnoreMe` struct and ignore it.
// Even though we constructed the instance, we don't need to unpack it.
fun test_ignore() {
    let no_drop = NoDrop {};
    let _ = IgnoreMe { a: 1, b: 2 }; // no need to unpack

    // The value must be unpacked for the code to compile.
    let NoDrop {} = no_drop; // OK
}
```

The drop `ability` is commonly applied to custom collection types to avoid the need for explicit cleanup when the collection is no longer required. For instance, the `vector` type includes the `drop` ability, which allows a `vector` to be ignored without further handling. The most distinctive aspect of Move's type system, however, is that types can be defined without `drop`. This guarantees that assets must be explicitly managed and cannot be silently ignored.

<!--TODO: check witness pattern-->

## Types with drop ability

All native types in Move have the `drop` ability. This includes [primitive types](./primitive_types.md) like `u8`, `u16`, `u32`, `u64`, `u128`, `u256`, `bool`, and [`address`](./address_type.md), as well as `vector<T>` (when `T` has drop).

Standard library types such as `Option<T>` (when `T` has `drop`) and `String` have `drop` as well.


<!--TODO: check Type Name-->
