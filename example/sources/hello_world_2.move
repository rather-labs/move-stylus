module hello_world::hello_world_2;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::event::emit;
use hello_world::stack::Stack;
use hello_world::stack;

use hello_world::other_mod::{generic_identity, generic_identity_two_types};

// Usage of generic functions
public fun echo_with_generic_function_u16(x: u16): u16 {
    generic_identity(x)
}

public fun echo_with_generic_function_vec32(x: vector<u32>): vector<u32> {
    generic_identity(x)
}

public fun echo_with_generic_function_u16_vec32(x: u16, y: vector<u32>): (u16, vector<u32>) {
    generic_identity_two_types(x, y)
}

public fun echo_with_generic_function_address_vec128(x: address, y: vector<u128>): (address, vector<u128>) {
    generic_identity_two_types(x, y)
}

public fun get_fresh_object_address(ctx: &mut TxContext): address {
    ctx.fresh_object_address()
}

public fun get_unique_ids(ctx: &mut TxContext): (UID, UID, UID) {
    (
        object::new(ctx),
        object::new(ctx),
        object::new(ctx),
    )
}

public fun get_unique_id(ctx: &mut TxContext): UID {
    object::new(ctx)
}

// Events

public struct TestEvent1 has copy, drop {
    n: u32
}

public struct TestEvent2 has copy, drop {
    a: u32,
    b: vector<u8>,
    c: u128,
}

public struct TestEvent3 has copy, drop {
    a: TestEvent1,
    b: TestEvent2,
}

public struct TestGenericEvent<T, U, V> has copy, drop {
    o: T,
    p: U,
    q: V,
}

public struct TestGenericEvent2<T, U, V> has copy, drop {
    o: T,
    p: U,
    q: V,
    r: vector<T>,
    s: TestGenericEvent<T, U, V>,
}

public fun emit_test_event1(n: u32) {
    emit(TestEvent1 { n });
}

public fun emit_test_event2(a: u32, b: vector<u8>, c: u128) {
    emit(TestEvent2 { a, b, c });
}

public fun emit_test_event3(a: TestEvent1, b: TestEvent2) {
    emit(TestEvent3 { a, b });
}

public fun emit_test_event_generic_1(o: u32, p: bool, q: TestEvent1) {
    emit(TestGenericEvent { o, p, q });
}

public fun emit_test_event_generic_2(o: u32, p: bool, q: TestEvent1, r: vector<u32>) {
    let s = TestGenericEvent {o, p, q};
    emit(TestGenericEvent2 { o, p, q, r, s });
}


public fun test_stack_1(): (Stack<u32>, u64) {
    let mut s = stack::new(vector[1, 2, 3]);
    s.push_back(5);
    s.push_back(6);
    (s, s.size())
}

public fun test_stack_2(): (Stack<u32>, u64){
    let mut s = stack::new(vector[]);
    s.push_back(5);
    s.push_back(6);
    (s, s.size())
}

public fun test_stack_3(): (Stack<u32>, u64){
    let mut s = stack::new(vector[3,1,4,1,5]);
    s.push_back(5);
    s.push_back(6);
    s.pop_back();
    s.pop_back();
    (s, s.size())
}
