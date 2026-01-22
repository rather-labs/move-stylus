module hello_world::counter;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}, 
    transfer::{Self}
};

#[test_only]
use stylus::test_scenario;

public struct Counter has key {
    id: UID,
    owner: address,
    value: u64
}

entry fun create(ctx: &mut TxContext) {
  transfer::share_object(Counter {
    id: object::new(ctx),
    owner: ctx.sender(),
    value: 25
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

    increment(&mut c);
    assert!(c.value == 1);

    test_scenario::drop_storage_object(c);
}

#[test]
fun test_read() {
    let mut ctx = test_scenario::new_tx_context();
    let uid = object::new(&mut ctx);
    let c = Counter { id: uid, owner: @0x2, value: 42 };

    let v = read(&c);
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

    set_value(&mut c, 99, &ctx);

    assert!(c.value == 99);

    test_scenario::drop_storage_object(c);
}

#[test, expected_failure]
fun test_set_value_wrong_owner_should_fail() {
    test_scenario::set_sender_address(@0x5);
    let mut ctx = test_scenario::new_tx_context();
    let uid = object::new(&mut ctx);
    let mut c = Counter { id: uid, owner: @0x4, value: 5 };


    set_value(&mut c, 99, &ctx);

    assert!(c.value == 99);

    test_scenario::drop_storage_object(c);
}
