module test::events_anon_2;

use stylus::event::emit;

public struct NestedStruct has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

#[ext(event, anonymous, indexes = 3)]
public struct TestEvent1Anon has copy, drop {
    a: u32,
    b: address,
    c: NestedStruct,
}

#[ext(event, anonymous, indexes = 3)]
public struct TestEvent2Anon has copy, drop {
    a: u32,
    b: vector<u8>,
    c: NestedStruct,
}

#[ext(event, anonymous, indexes = 1)]
public struct TestEvent3Anon has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[ext(event, anonymous, indexes = 2)]
public struct TestEvent4Anon has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[ext(event, anonymous, indexes = 4)]
public struct Anonymous has copy, drop {
    a: u32,
    b: u128,
    c: vector<u8>,
    d: NestedStruct,
}

entry fun emit_test_anon_event1(a: u32, b: address, c: u32, d: address, e: u128) {
    let c = NestedStruct {a: c, b: d, c: e };
    emit(TestEvent1Anon { a, b, c });
}

entry fun emit_test_anon_event2(a: u32, b: vector<u8>, c: u32, d: address, e: u128) {
    let c = NestedStruct {a: c, b: d, c: e };
    emit(TestEvent2Anon { a, b, c });
}

entry fun emit_test_anon_event3(a: u64, b: std::ascii::String) {
    emit(TestEvent3Anon { a, b });
}

entry fun emit_test_anon_event4(a: u64, b: std::ascii::String) {
    emit(TestEvent4Anon { a, b });
}

entry fun emit_test_anonymous(a: u32, b: u128, c: vector<u8>, d: u32, e: address, f: u128) {
    let d = NestedStruct {a: d, b: e, c: f };
    emit(Anonymous { a, b, c, d });
}
