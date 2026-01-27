# Cross contract calls

In smart contract development, cross contract calls refer to the ability of one smart contract to invoke functions or methods of another smart contract. This feature is essential for building complex decentralized applications (dApps) that require interaction between multiple contracts.

In Move, to be able to perform cross contract calls, first you need to define a module that contains the functions you want to call from another contract. Let's use the [ERC-20](https://ethereum.org/developers/docs/standards/tokens/erc-20/) token standard as an example.

```move
module erc20call::erc20call;

use stylus::contract_calls::{ContractCallResult, CrossContractCall};

#[ext(external_call)]
public struct ERC20(CrossContractCall) has drop;

public fun new(configuration: CrossContractCall): ERC20 {
    ERC20(configuration)
}

#[ext(external_call, view)]
public native fun total_supply(self: &ERC20): ContractCallResult<u256>;

#[ext(external_call, view)]
public native fun balance_of(self: &ERC20, account: address): ContractCallResult<u256>;

#[ext(external_call)]
public native fun transfer(self: &ERC20, account: address, amount: u256): ContractCallResult<bool>;

#[ext(external_call, view)]
public native fun allowance(self: &ERC20, owner: address, spender: address): ContractCallResult<u256>;

#[ext(external_call)]
public native fun approve(self: &ERC20, spender: address, amount: u256): ContractCallResult<bool>;

#[ext(external_call)]
public native fun transfer_from(self: &ERC20, sender: address, recipient: address, amount: u256): ContractCallResult<bool>;
```

In the next sections, we will explore all the components involved in the snippet above.

## The `CrossContractCall` Struct

The `CrossContractCall` struct is a key component that facilitates cross contract calls. It encapsulates the necessary information and configuration required to perform these calls. When you create an instance of the `ERC20` struct, you pass in a `CrossContractCall` configuration that specifies how to interact with the target contract.

The `ERC20` struct is the one that will be used to perform the cross contract calls to the _ERC-20_ contract. To be able to do that, it must meet the following requirements:

- It must be annotated with the `#[ext(external_call)]` attribute.

- It must be a tuple struct containing *only* a field of type `CrossContractCall`.

- It must have the `drop` ability.

### Creating a New Instance

To create a new instance of the cross contract call struct, you need to provide a `CrossContractCall` configuration. This configuration specifies the address of the target contract and any additional settings required for the cross contract calls.

- **`new(address)`**: This function creates a new `CrossContractCall` instance with the specified contract address. It initializes the configuration with default values for gas and value.

- **`gas(u64)`**: Amount of gas to send to the sub context to execute. The gas that is not used by the sub context is returned to this one.

- **`value(u256)`**: Value in WEI to send to the account.

- **`delegate()`**: This function configures the cross contract call to be a delegate call.

#### Example

```move
let erc20 = erc20call::new(
    ccc::new(erc20_address)
        .gas(100000)
        .value(0)
);
```

## Defining Cross Contract Call Functions

Each cross contract call function must follow the following requirements:

- It must be annotated with the `#[ext(external_call)]` attribute:
You can also add [function modifiers](./abi.md#function-modifiers) such as `view` or `pure` to indicate that the function does not modify the state. This will hint the compiler to [optimize the call accordingly](https://docs.soliditylang.org/en/latest/contracts.html#view-functions) by using a static call instead of a common one).

- It must have a `self` parameter of type `&ERC20` (or the name of the struct you defined to perform the cross contract calls). This is to associate the method to the struct that contains the `CrossContractCall` configuration.

- It must have the same parameters as the target function in the called contract.

- It must be declared as **`native`** since it will be implemented automatically by the compiler.

- It must return one of the following types:
    - `ContractCallResult<T>` where `T` is the return type of the target function in the called contract. This wrapper type is used to handle potential errors that may occur during the cross contract call.
    - `ContractCallEmptyResult` if the target function in the called contract does not return any value.

> [!NOTE]
> The cross contact calls follow the same ABI rules as the regular functions. i.e: If the target function contains a `ID` parameter, the type for that parameter will be `bytes32`.

### `ContractCallResult<T>` struct

The `ContractCallResult<T>` struct is a generic wrapper type used to encapsulate the result of a cross contract call. It provides methods to handle the result and potential errors that may occur during the call:

- `succeded`: Returns `true` if the cross contract call was successful, otherwise returns `false`.

    ```move
    let result: ContractCallResult<u256> = erc20.balance_of(address);
    if (result.succeded()) {
        let balance: u256 = result.get_result();
        // Use the balance
    } else {
        // Handle the error
    }
    ```

- `get_result`: Returns the actual result of the cross contract call if it was successful. If the call failed, this method will abort the transaction.

    ```move
    let result: ContractCallResult<u256> = erc20.total_supply();
    let total_supply: u256 = result.get_result();
    ```

### `ContractCallEmptyResult` struct

The `ContractCallEmptyResult` struct is used for cross contract calls that do not return any value. It provides a method to check if the call was successful:

- `succeded`: Returns `true` if the cross contract call was successful, otherwise returns `false`.

    ```move
    let result: ContractCallEmptyResult = cross_contract_call.call_some_function(123);
    if (result.succeded()) {
        // Approval succeeded
    } else {
        // Handle the error
    }
    ```

## Using Cross Contract Calls

To use the cross contract calls defined in the `erc20call` module, you need to create an instance of the `ERC20` struct with the appropriate `CrossContractCall` configuration. Then, you can invoke the methods defined in the struct to interact with the target ERC-20 contract.

```move
module book::erc20user;

use erc20call::erc20call::{Self};
use stylus::contract_calls as ccc;

entry fun balance_of_erc20(erc20_address: address, balance_address: address): u256 {
    let erc20 = erc20call::new(ccc::new(erc20_address));
    erc20.balance_of(balance_address).get_result()
}

entry fun total_supply(erc20_address: address): u256 {
    let erc20 = erc20call::new(ccc::new(erc20_address));
    erc20.total_supply().get_result()
}
```

## Delegated Calls

In addition to direct cross contract calls, Move also supports delegated calls. A delegated call allows a contract to execute code in the context of another contract, effectively allowing it to "borrow" the functionality of that contract. This is useful for scenarios where you want to extend the functionality of an existing contract without modifying its code.

To illustrate how delegated calls work, let's define four moodules:
- `delegated_counter`: A simple contract that maintains a counter and provides functions to increment and get the counter value via delegate calls.

- `delegated_counter_interface`: An interface module that defines the cross contract call structure for the `delegated_counter` contract (just like we did with the ERC-20 at the begginning of this chapter).

- `counter_logic_a`: A contract that contains logic to increment the counter by 1.

- `counter_logic_b`: A contract that contains logic to increment the counter by 2.

`counter`'s functions will be just proxy function using the delegate calls to call the logic defined in `counter_logic_a` and `counter_logic_b`.


> [!WARNING]
> Once a `CrossContractCall` object is maked to peform delegate calls, it cannot be undone and **all** the calls will be delegated.

#### Counter Module

```move
module book::delegated_counter;

use stylus::{
    tx_context::TxContext,
    object::{Self, UID},
    transfer::{Self},
    contract_calls::{Self}
};
use book::delegated_counter_interface as dci;

public struct Counter has key {
    id: UID,
    owner: address,
    value: u64,
    contract_address: address,
}

/// Create a new counter.
entry fun create(contract_logic: address, ctx: &mut TxContext) {
  transfer::share_object(Counter {
    id: object::new(ctx),
    owner: ctx.sender(),
    value: 25,
    contract_address: contract_logic,
  });
}

/// Increment a counter.
entry fun increment(counter: &mut Counter) {
    let delegated_counter = dci::new(
        contract_calls::new(counter.contract_address)
            .delegate()
    );
    let res = delegated_counter.increment(&mut counter.id);
    assert!(res.succeded(), 33);
}

/// Read counter.
entry fun read(counter: &Counter): u64 {
    counter.value
}

/// Change the address where the delegated calls are made.
entry fun change_logic(counter: &mut Counter, logic_address: address) {
    counter.contract_address = logic_address;
}
```

In the `increment` function, we create a `CrossContractCall` object configured for delegate calls using the `delegate()` method. We then call the `increment` function defined in the logic contract.

#### Delegated Counter Interface Module

```move
module book::delegated_counter_interface;

use stylus::{
    contract_calls::{ContractCallEmptyResult, CrossContractCall},
    object::UID
};

#[ext(external_call)]
public struct CounterCall(CrossContractCall) has drop;

public fun new(configuration: CrossContractCall): CounterCall {
    CounterCall(configuration)
}

#[ext(external_call)]
public native fun increment(self: &CounterCall, counter: &mut UID): ContractCallEmptyResult;
```

#### Counter Logic A Module

```move
module book::delegated_counter_logic_a;

use stylus::{
    tx_context::TxContext,
    object::UID
};

#[ext(external_struct, module_name = b"delegated_counter", address = @0x0)]
public struct Counter has key {
    id: UID,
    owner: address,
    value: u64,
    contract_address: address,
}

/// Increment a counter by 1.
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}
```

#### Counter Logic B Module

```move
module book::delegated_counter_logic_b;

use stylus::{
    tx_context::TxContext,
    object::UID
};

#[ext(external_struct, module_name = b"delegated_counter", address = @0x0)]
public struct Counter has key {
    id: UID,
    owner: address,
    value: u64,
    contract_address: address,
}

/// Increment a counter by 2.
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 2;
}
```

Once you have defined these modules, you should deploy the `delegated_counter_logic_a` and `delegated_counter_logic_b` modules. Then, when creating a new counter using the `create` function in the `delegated_counter` module, you can specify which logic contract to use for incrementing the counter.

Later, you can change the logic contract by calling the `change_logic` function, allowing you to switch between different incrementing behaviors dynamically.

> [!WARNING]
> Since delegated calls execute code in the context of the calling contract, that means that the calling contract's storage is the one that is modified.
>
> When interacting with objects from a caller conntrct, you **must** specify the module name and address where the object is defined using the `#[ext(external_struct, module_name = ..., address = ...)]` attribute. That is because objects with the same name can be defined in different modules. Those objects are different by definition and the Move compiler needs to know which one to use.
>
> If you don't specify the module name and address, the object you are trying to interact will not be found and a runtime error will be thrown.
