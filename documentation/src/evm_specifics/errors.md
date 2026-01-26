# Errors & Revert

In the Stylus framework, errors are represented as Move structs. To use a struct as a revert reason, it must be annotated with the `#[ext(abi_error)]` attribute.

## Revert
The framework provides a native `revert` function to halt execution and undo all state changes, returning the abi-encoded error to the caller.

```move 
module stylus::error;

/// Reverts the current transaction.
///
/// This function reverts the current transaction with a given error.
public native fun revert<T: copy + drop>(error: T);
```

The `revert` function is generic over the type `T`. For a successful compilation, `T` must be a struct annotated with `#[ext(abi_error)]`. 

## Error encoding

The framework ensures that Move errors follow the Solidity ABI specification. This allows Ethereum-compatible tools like Etherscan to decode and display the error correctly.

The **error selector** is a 4-byte identifier that tells the EVM which specific custom error is being triggered. It is calculated by taking the first 4 bytes of the `Keccak256` hash of the error's signature string. The signature string is composed of the error struct name followed by the field types in parentheses.

```move 
#[ext(abi_error)]
public struct ExampleError {
    message: String,
    code: u8
}
```
For the struct above, the signature string is `ExampleError(string,uint8)`. The selector is derived as: `keccak256("ExampleError(string,uint8)")` → `0x...` (first 4 bytes).

**The complete error message consists of the 4-byte selector followed by the ABI-encoded fields of the struct.**

>[!Tip]
The signature `Error(string)` is a special, built-in Solidity error type. If you define a struct that results in this signature, the Stylus node and most Ethereum explorers will automatically decode and display the raw string message.