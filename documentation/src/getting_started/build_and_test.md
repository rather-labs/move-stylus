# Build and Test

Before building anything, we must implement our contract logic. Copy the following code into the `sources/counter.move` file created in the previous step:

```move
module counter::counter;

use stylus::{
    tx_context::TxContext,
    object::{Self, UID},
    transfer::{Self}
};

#[test_only]
use stylus::test_scenario;

/// Initial value for new counters.
const INITIAL_VALUE: u64 = 1;

/// A simple counter object.
public struct Counter has key {
    id: UID,
    owner: address,
    value: u64
}

/// Create a new counter with initial value.
entry fun create(ctx: &mut TxContext) {
  transfer::share_object(Counter {
    id: object::new(ctx),
    owner: ctx.sender(),
    value: INITIAL_VALUE,
  });
}

/// Increment a counter by 1.
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}


/// Read counter.
#[ext(abi(view))]
entry fun read(counter: &Counter): u64 {
    counter.value
}

/// Set value (only runnable by the Counter owner)
entry fun set_value(counter: &mut Counter, value: u64, ctx: &TxContext) {
    assert!(counter.owner == ctx.sender(), 0);
    counter.value = value;
}

//
// Unit tests
//
#[test]
fun test_increment() {
    let mut ctx = test_scenario::new_tx_context();
    let uid = object::new(&mut ctx);
    let mut c = Counter { id: uid, owner: @0x1, value: 0 };

    c.increment();
    assert!(c.value == 1);

    test_scenario::drop_storage_object(c);
}

#[test]
fun test_read() {
    let mut ctx = test_scenario::new_tx_context();
    let uid = object::new(&mut ctx);
    let c = Counter { id: uid, owner: @0x2, value: 42 };

    let v = c.read();
    assert!(v == 42);

    test_scenario::drop_storage_object(c);
}

#[test]
fun test_set_value_by_owner() {
    let mut ctx = test_scenario::new_tx_context();
    let uid = object::new(&mut ctx);

    let mut c = Counter {
        id: uid,
        owner: test_scenario::default_sender(),
        value: 5
    };

    c.set_value(99, &ctx);

    assert!(c.value == 99);

    test_scenario::drop_storage_object(c);
}

#[test, expected_failure]
fun test_set_value_wrong_owner_should_fail() {
    test_scenario::set_sender_address(@0x5);
    let mut ctx = test_scenario::new_tx_context();
    let uid = object::new(&mut ctx);
    let mut c = Counter { id: uid, owner: @0x4, value: 5 };


    c.set_value(99, &ctx);

    assert!(c.value == 99);

    test_scenario::drop_storage_object(c);
}
```

There are a lot of new concepts in this code that we will cover in future sections. For now, just note that we have defined a simple counter contract with the ability to create, increment, read, and set the value of a counter. We have also included some unit tests to verify the functionality of our contract.

## Building the Project

Now that we have our contract code in place, we can build the project using the `move-stylus` CLI. Open a terminal, navigate to the root directory of your project (`counter`), and run the following command:

```bash
move-stylus build
```

You should see output indicating that the build was successful:

```bash
INCLUDING DEPENDENCY StylusFramework
INCLUDING DEPENDENCY MoveStdlib
BUILDING counter
COMPILING counter
```

After building, the code is ready to be deployed to a blockchain or tested locally. We will cover deployment in a later section.

## Running Tests

To ensure that our contract works as expected, we can run the unit tests we defined earlier. Use the following command to run the tests:

```bash
move-stylus test
```

You should see output indicating that all tests have passed:

```bash
COMPILING counter

Running 0x0::counter tests (./sources/counter.move)

  0x0::counter::test_increment ... PASSED
  0x0::counter::test_read ... PASSED
  0x0::counter::test_set_value_by_owner ... PASSED
  0x0::counter::test_set_value_wrong_owner_should_fail [expected failure] ... PASSED

Total Tests : 4, Passed: 4, Failed: 0.
```
