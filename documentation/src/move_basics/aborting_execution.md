# Aborting Execution

A transaction Move can either succeed or fail. When execution succeeds, all modifications to on‑chain data are applied, and the transaction is committed to the blockchain. If execution aborts, none of the changes are preserved. The `abort` keyword and `revert` function from the Stylus Framework are used to terminate a transaction and revert any modifications that were made.

> [!NOTE]
> It is important to understand that Move does not provide a catch mechanism. When a transaction aborts, all changes performed up to that point are rolled back, and the transaction is marked as failed.

## Abort

The `abort` keyword is used to terminate execution immediately. It must be used with an error code. The abort code is a `u64` value.

```move
let user_has_access = false;

// Abort with error code 1
if (!user_has_access) {
    abort 1;
}
```

### Error Constants

Defining error constants is a good practice for making error codes more descriptive. These constants are declared using `const` and are typically prefixed with `E` followed by a `UpperCamelCase` name. Error constants behave like any other constants and do not receive special treatment. Their main purpose is to enhance code readability and make abort scenarios easier to interpret.

```move
const EUserNotAuthorized: u64 = 1;
let user_has_access = false;

// Abort with error code 1
if (!user_has_access) {
    abort EUserNotAuthorized;
}
```

## assert!

The `assert!` macro is a convenient way to check a condition and abort execution if the condition is not met. It takes a boolean expression and an optional error code. If the expression evaluates to `false`, the transaction aborts with the specified error code (or a default code if none is provided).

```move
let user_has_access = false;

// Assert that the user has access, aborting with error code 2 if not
assert!(user_has_access, 2);
```

<!-- TODO: Add when added compatibility
# Error constants
!-->

## Custom error structs

Move allows you to define custom [structures](./structs.md) to represent errors. This approach provides more context about the error and can include additional information beyond a simple error code. The errors raised using these structs follows the [Solidity's errors ABI](https://docs.soliditylang.org/en/latest/abi-spec.html#errors), meanning that they can be docoded by any external tools that understand it.

To be able to use a struct as an error, it must be annotated with the `#[ext(abi_error)]` attribute. This attribute indicates that the struct is intended to be used as an external ABI error.

```move
#[ext(abi_error)]
public struct CustomError has copy, drop {
    error_message: String,
    error_code: u64,
}

public fun revert_custom_error(s: String, code: u64) {
    revert( CustomError { error_message: s, error_code: code });
}
```
