# Address

`Address` is a unique identifier of a location on the blockchain. It is used to identify contracts, accounts, and objects. `Address` has a fixed size of 32 bytes and is usually represented as a hexadecimal string prefixed with `0x`. Addresses are case insensitive.

```
0xe51ff5cd221a81c3d6e22b9e670ddf99004d71de4f769b0312b68c7c4872e2f1
```

The address above is an example of a valid address. It is 64 characters long (32 bytes) and prefixed with `0x`.

Move also has reserved addresses that are used to identify standard packages and objects. Reserved addresses are typically simple values that are easy to remember and type. For example, the address of the Standard Library is `0x1`. Addresses, shorter than 32 bytes, are padded with zeros to the left.

Here are some examples of reserved addresses:

* `0x1` - address of the Sui Standard Library (alias `std`)
* `0x2` - address of the Stylus Framework (alias `stylus`)

## Comparison with Solidity (EVM)

If you are coming from an Ethereum background, it is important to note two key differences:

1. Size: Move addresses are 32 bytes, whereas Solidity addresses are 20 bytes.

2. Padding & Alignment: In the EVM, addresses are often "padded" to 32 bytes during ABI encoding for word alignment, but the underlying identity is only 20 bytes. In Sui, the full 32 bytes represent the actual identity of the account or object.

To maintain interoperability, Move addresses can be thought as 32-byte "containers" where only the least significant 20 bytes hold the actual EVM address. The most significant 12 bytes are strictly left-padded with zeros.
