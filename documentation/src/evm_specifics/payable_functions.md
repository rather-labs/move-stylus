# Payable Functions

In Solidity, a payable function is a special type of function that can receive Ether. When a function is marked as `payable`, it allows the contract to accept and process incoming Ether transactions.

In Move, the amount of WEI sent with a transaction can be accessed using the `TxContext` struct from the Stylus Framework. This struct provides a method called `value()` that returns the amount of WEI sent with the message.

Payable functions do not require any special declaration in Move. Any entry function can access the value sent with the transaction through the `TxContext`. Here is an example of a payable function in Move:

```move
use stylus::tx_context::{Self};

#[abi(payable)]
entry fun deposit_funds(ctx: &TxContext) {
    let amount = ctx.value();
    // Process the received amount as needed
}
```

In the example above, the `deposit_funds` function is an entry function that can receive WEI. The amount of WEI sent with the transaction is accessed using the `ctx.value()` method. The `#[abi(payable)]` attribute indicates that this function is intended to be payable, useful when exporting the contract's ABI.

