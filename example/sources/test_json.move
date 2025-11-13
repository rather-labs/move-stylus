module hello_world::test_json;

use stylus::event::emit;

#[ext(event, indexes = 2)]
public struct TestEvent1 has copy, drop {
    from: address,
    to: address,
    value: u256
}

public struct EventField has copy, drop {
    a: u32,
    b: u64,
    c: vector<u128>,
}

public struct EventField2 has copy, drop {
    a: EventField,
    b: u32,
}

#[ext(event, indexes = 2)]
public struct TestEvent2 has copy, drop {
    from: address,
    to: address,
    struct_field: EventField,
    struct_field2: EventField2,
}

entry fun emit_test_event1(from: address, to: address, value: u256) {
    emit(TestEvent1 { from, to, value });
}

entry fun emit_test_event2(from: address, to: address, struct_field: EventField, struct_field2: EventField2) {
    emit(TestEvent2 { from, to, struct_field, struct_field2 });
}
