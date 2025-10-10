module test::event;

use stylus::event::emit;

#[ext(event, indexes = 1)]
public struct TestEvent1 has copy, drop {
    n: u32
}

#[ext(event, indexes = 3)]
public struct TestEvent2 has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

#[ext(event, indexes = 2)]
public struct TestEvent3 has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
}

#[ext(event, indexes = 2)]
public struct TestEvent4 has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
    e: TestEvent2,
}

public fun emit_test_event1(n: u32) {
    emit(TestEvent1 { n });
}

public fun emit_test_event2(a: u32, b: address, c: u128) {
    emit(TestEvent2 { a, b, c });
}

public fun emit_test_event3(a: u32, b: address, c: u128, d: vector<u8>) {
    emit(TestEvent3 { a, b, c, d });
}

public fun emit_test_event4(a: u32, b: address, c: u128, d: vector<u8>, e: TestEvent2) {
    emit(TestEvent4 { a, b, c, d, e });
}
