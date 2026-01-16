module test::events_2;

use stylus::event::emit;

#[allow(unused_field)]
public struct NestedStruct has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

#[ext(event(indexes = 2))]
public struct TestEvent1 has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
    e: NestedStruct,
}

#[ext(event(indexes = 3))]
public struct TestEvent2 has copy, drop {
    a: u32,
    b: address,
    c: NestedStruct,
}

#[ext(event(indexes = 3))]
public struct TestEvent3 has copy, drop {
    a: u32,
    b: vector<u8>,
    c: vector<NestedStruct>,
}

#[ext(event(indexes = 1))]
public struct TestEvent4 has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[ext(event(indexes = 2))]
public struct TestEvent5 has copy, drop {
    a: u64,
    b: std::ascii::String,
}

entry fun emit_test_event1(a: u32, b: address, c: u128, d: vector<u8>, e: NestedStruct) {
    emit(TestEvent1 { a, b, c, d, e });
}

entry fun emit_test_event2(a: u32, b: address, c: NestedStruct) {
    emit(TestEvent2 { a, b, c });
}

entry fun emit_test_event3(a: u32, b: vector<u8>, c: vector<NestedStruct>) {
    emit(TestEvent3 { a, b, c });
}

entry fun emit_test_event4(a: u64, b: std::ascii::String) {
    emit(TestEvent4 { a, b });
}

entry fun emit_test_event5(a: u64, b: std::ascii::String) {
    emit(TestEvent5 { a, b });
}
