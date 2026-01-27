# Signer Type


`signer` is a built-in Move type. A signer represents a capability that allows its holder to act on behalf of a specific address. Conceptually, the native implementation can be thought of as:

```move
struct signer has drop { a: address }
```

A signer holds the address which signed the transaction being executed.

## Comparison to `address`

A Move program can freely create any `address` value without special permission by using address literals:

```move
let a1 = @0x1;
let a2 = @0x2;
// ... and so on for every other possible address
```

However, creating a `signer` value is restricted. A Move program cannot arbitrarily create a `signer` for any address. Instead, a `signer` can only be obtained through specific entry functions that are invoked as part of a transaction signed by the corresponding address.

```move
entry fun example(s: signer) {
    /// Do something with signer
}
```

> [!WARNING]
> Only one signer can be passed to an entry function, representing the address that signed the transaction. Attempting to pass multiple signers or create signers for arbitrary addresses will result in a compilation error.
