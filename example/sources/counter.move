module hello_world::hello_world;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;
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
    let mut c = Counter { id: uid, owner: @0x3, value: 5 };

    set_value(&mut c, 99, &ctx);

    assert!(c.value == 99);

    test_scenario::drop_storage_object(c);
}

/*
#[test]
fun test_set_value_wrong_owner_should_fail() {
    let ctx_owner = test_scenario::new_tx_context(@0x4);
    let uid = object::new(&mut ctx_owner);
    let mut c = Counter { id: uid, owner: @0x4, value: 5 };

    // Now simulate a different sender
    let ctx_attacker = test_scenario::new_tx_context(@0x5);

    // This should trigger the assert and abort
    // (unit test framework expects aborts to be caught)
    let result = test_scenario::catch_abort(|| {
        set_value(&mut c, 123, &ctx_attacker);
    });
    assert!(result.is_err());
}
*/
