module test::event;

use stylus::event::emit;

public struct TestEvent1 has copy, drop {
    n: u32
}

public struct TestEvent2 has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

public struct TestEvent3 has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
}

public struct TestEvent4 has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
    e: TestEvent2,
}

public struct TestEventGeneric<T, U, V> has copy, drop {
    n: T,
    o: bool,
    p: U,
    q: V,
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

public fun emit_generic_event_1(n: vector<u32>, o: bool, p: address, q: u128) {
    emit(TestEventGeneric { n, o, p, q })
}

public fun emit_generic_event_2(n: u64, o: bool, p: TestEvent1, q: TestEvent2) {
    emit(TestEventGeneric { n, o, p, q })
}