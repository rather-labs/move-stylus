# Constructor

The `init` function is a specialized entry point used to run logic exactly once during module deployment. This is the primary way to initialize global state, set up configurations, or mint initial objects.

## Requirements

To be recognized as a *constructor*, a function must meet the following criteria:

1.  **Naming**: The function name must be exactly `init`.
2.  **Visibility**: The function must be `private`.
3.  **Signature**: It takes a single argument, a reference to the [`TxContext`](./transaction_context.md), and has no return values.
4.  **Exclusivity**: There can only be one `init` function per module.

>[!Important]
If any of the above requirements is not met, the compiler will throw an error.

## Implementation Example

The following snippet demonstrates a classic `init` implementation. When the module is deployed, it creates and shares a `Foo` object.

```move
module test::constructor;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}, 
    transfer::{Self}
};

public struct Foo has key {
    id: UID,
    value: u64
}

entry fun init(ctx: &mut TxContext) {
  let foo = Foo {
    id: object::new(ctx),
    value: 101,
  };

  transfer::share_object(foo);
}
```

## Technical Enforcement

The framework ensures the "once-only" execution of the `init` function through a persistent storage flag.

* During compilation, a deterministic storage slot is assigned to the initialization flag. This slot is calculated as the `Keccak256` hash of the string "init_key".

* Upon deployment, the runtime checks this storage slot. If the flag is `false`, the `init` function executes. Once successful, the flag is set to `true`.

* This prevents anyone from re-triggering the initialization logic after the contract is live, ensuring the module's setup and configuration remain immutable.