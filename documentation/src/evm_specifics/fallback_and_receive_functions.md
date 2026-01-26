# Receive & Fallback Functions

`Receive` and `Fallback` are special-purpose functions triggered by indirect interactions—scenarios where a contract is called without a specific function identifier. While these are unique to the EVM and not native to Move, the framework provides a bridge that allows developers to implement this logic directly in Move modules.

## Receive Function

In the EVM ecosystem, the `receive` function is a dedicated gateway for plain Ether transfers (calls with empty calldata). According to the Solidity specification:

>A contract can have at most one receive function, declared using `receive() external payable { ... }` (without the function keyword). This function cannot have arguments, cannot return anything and must have external visibility and payable state mutability. It can be virtual, can override and can have modifiers.

To implement this in Move, targeting Arbitrum Stylus, a function must meet the following requirements:

1. **Naming**: The function name must be exactly `receive`.
2. **Visibility**: It must be marked as an `entry` function.
3. **Signature**: It must take no arguments (except a reference to the `stylus::TxContext`) and return no values.
4. **State Mutability**: It must be annotated with the `#[ext(abi(payable))]` attribute to accept ether.

```move
use stylus::tx_context::TxContext;

#[ext(abi(payable))]
entry fun receive(ctx: &TxContext) {
  // Custom logic
}
```

> [!IMPORTANT]
The `receive` function is executed on a call to the contract with **empty calldata**. This is the function that is executed on plain Ether transfers. If no such function exists, but a payable `fallback` function exists, the `fallback` function will be called on a plain Ether transfer. If neither a `receive` Ether nor a payable `fallback` function is present, the contract cannot receive Ether through a transaction that does not represent a payable function call and throws an exception.

## Fallback Function

The `fallback` function serves as the "catch-all" handler for a contract. According to the Solidity specification:

> A contract can have at most one fallback function, declared using either `fallback () external [payable]` or `fallback (bytes calldata input) external [payable] returns (bytes memory output)` (both without the function keyword). This function must have external visibility. A fallback function can be virtual, can override and can have modifiers.

In the Stylus framework, implementing a `fallback` function requires adhering to these specific rules:

1. **Naming**: The function name must be exactly `fallback`.
2. **Visibility**: It must be marked as an `entry` function.
3. **State Mutability**: It may optionally be annotated with `#[ext(abi(payable))]` if it needs to accept Ether.
4. **Signature**: It takes no arguments, except optionally a reference to the [`TxContext`](./transaction_context.md), and _can_ return a `vector<u8>` representing some raw output bytes **without abi-encoding**.

For instance, both of these are valid `fallback` declarations:

```move
#[ext(abi(payable))]
entry fun fallback(ctx: &TxContext): vector<u8> { 
    // Do something with the calldata
}

entry fun fallback() {}
```

The `fallback` function is triggered in two main cases:
* **Unknown Selectors**: When a caller attempts to invoke a function signature that does not exist in the module.
* **Empty Data**: When a contract receives a call with no data and no `receive` function is defined.
