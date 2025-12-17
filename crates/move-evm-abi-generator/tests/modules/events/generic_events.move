module test::generic_events;

use stylus::event::emit;

#[allow(unused_field)]
public struct NestedStruct<T,U> has copy, drop {
    a: T,
    b: U,
    c: vector<T>,
    d: vector<U>,
}

#[ext(event, indexes = 2)]
public struct TestEvent1<T,U,V> has copy, drop {
    a: T,
    b: NestedStruct<U,V>,
}

entry fun test_event_u16_u32_u64(a: u16, b: NestedStruct<u32,u64>) {
    let event = TestEvent1<u16,u32,u64> { a, b };
    emit(event);
}

entry fun test_event_address_bool_u256(a: address, b: NestedStruct<bool,u256>) {
    let event = TestEvent1<address,bool,u256> { a, b };
    emit(event);
}
