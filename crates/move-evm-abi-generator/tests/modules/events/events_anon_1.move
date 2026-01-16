module test::events_anon_1;

use stylus::event::emit;

public struct NestedStruct has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

#[ext(event(anonymous, indexes = 1))]
public struct TestEvent1Anon has copy, drop {
    n: u32
}

#[ext(event(anonymous, indexes = 3))]
public struct TestEvent2Anon has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

#[ext(event(anonymous, indexes = 2))]
public struct TestEvent3Anon has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
}

#[ext(event(anonymous, indexes = 2))]
public struct TestEvent4Anon has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
    e: NestedStruct,
}

#[ext(event(anonymous, indexes = 3))]
public struct TestEvent5Anon has copy, drop {
    a: u32,
    b: address,
    c: vector<u8>,
}

entry fun emit_test_anon_event1(n: u32) {
    emit(TestEvent1Anon { n });
}

entry fun emit_test_anon_event2(a: u32, b: address, c: u128) {
    emit(TestEvent2Anon { a, b, c });
}

entry fun emit_test_anon_event3(a: u32, b: address, c: u128, d: vector<u8>) {
    emit(TestEvent3Anon { a, b, c, d });
}

entry fun emit_test_anon_event4(a: u32, b: address, c: u128, d: vector<u8>, e: u32, f: address, g: u128) {
    let e = NestedStruct {a: e, b: f, c: g };
    emit(TestEvent4Anon { a, b, c, d, e });
}

entry fun emit_test_anon_event5(a: u32, b: address, c: vector<u8>) {
    emit(TestEvent5Anon { a, b, c });
}

