# Address Type


Move uses a special type called `address` to represent blockchain addresses. It is a 32-byte value capable of representing any address on the chain.

Addresses can be written in two forms:
- **Hexadecimal addresses** prefixed with `0x`
- **Named addresses** defined in `Move.toml`

```move
// address literal
let value: address = @0x1;

// named address registered in Move.toml
let value = @std;
let other = @stylus;
```

An address literal begins with the `@` symbol followed by either a hexadecimal number or an identifier:

- The hexadecimal number is interpreted as a 32-byte value.
- The identifier is resolved in the `Move.toml` file and replaced with the corresponding address by the compiler.

If the identifier is not found in `Move.toml`, the compiler will throw an error.

## Address Length

In EVM, the blockchain addresses are typically 20 bytes long. However, Move's `address` type is 32 bytes long to ensure compatibility.

When working with EVM addresses in Move, it is common to use the lower 20 bytes of the 32-byte `address` type. The higher 12 bytes are usually set to zero.

For example, the [`UID`](../object_model/uid_and_id.md) types internally contains an `address`:

```move
/// References a object ID
public struct ID has copy, drop, store {
    bytes: address,
}

/// Globally unique IDs that define an object's ID in storage. Any object, that is a struct
/// with the `key` ability, must have `id: UID` as its first field.
public struct UID has store {
    id: ID,
}
```

Since addresses are 32 bytes long, the `UID` type can represent any object ID in the Move storage system.

> [!WARNING]
> When interacting with EVM contracts, it is important to ensure that the addresses are correctly formatted and that only the lower 20 bytes are present.
>
> i.e: ``` let address: address = @0x1234567890abcdef1234567890abcdef12345678; ```
