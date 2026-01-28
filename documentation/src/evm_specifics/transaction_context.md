# Transaction Context

In Stylus, the transaction context provides essential information about the transaction being executed. This context includes details such as the sender's address, the value transferred, gas limit, and other relevant metadata.

To access the transaction context in your Move modules, you can use the `TxContext` struct provided by the [Stylus Framework](./../stylus_framework). This struct encapsulates all the information about the executing transaction.


> [!NOTE]
> The `TxContext` struct is a way to access transaction-specific information accessed using [global variables in Solidity](https://docs.soliditylang.org/en/latest/units-and-global-variables.html#block-and-transaction-properties).

The information is accesed through `TxContext`'s [struct methods](../move_basics/struct_methods.md):

- **`sender`**: Return the address of the user that signed the current transaction.
- **`value`**: Return the number of WEI sent with the message.
- **`block_number`**: Return the current block's number.
- **`block_basefee`**: Return the current block's base fee (EIP-3198 and EIP-1559).
- **`block_gas_limit`**: Return the current block's gas limit.
- **`block_timestamp`**: Return the current block's timestamp as seconds since unix epoch.
- **`chain_id`**: Return the chain ID of the current transaction.
- **`gas_price`**: Return the gas price of the transaction.
- **`data`**: Return the calldata of the current transaction as a `vector<u8>`.

## Using TxContext

The `TxContext` struct is an special struct handled entirely by the compiler. This means that an instance of the object cannot be created directly. Instead, you can obtain a reference (muttable or immutable) to the current transaction context by declaring it as a parameter in an `entry` function.

```move
use stylus::tx_context::{Self};

entry fun example_function(tx: &TxContext) {
    let sender_address = tx.sender();
    let value_sent = tx.value();
    // You can now use sender_address and value_sent as needed
}
```

In the example above, the `example_function` entry function takes a reference to the `TxContext` as a parameter. Inside the function, you can access various properties of the transaction context using the provided methods.

## ABI

Any appearence of the `TxContext` struct in the function signature will be omitted from the generated ABI, as it is an implicit parameter provided by the execution environment. For example, in the `set_value` function extracted from the [Build and Test](../getting_started/build_and_test.md) guide:

```move
/// Set value (only runnable by the Counter owner)
entry fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    assert!(counter.owner == ctx.sender(), 0);
    counter.value = value;
}
```

The transaction context is used to assert that the sender of the transaction is the owner of the `Counter` resource. However, in the generated ABI, the `ctx` parameter will not be included, as it is implicitly provided by the Stylus execution environment.

As seen in the [Abi](./abi.md) section, the ABI for the `set_value` function is:

```solidity
function setValue(bytes32 counter, uint64 value) external;
```
