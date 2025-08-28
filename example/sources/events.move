module hello_world::events;

use stylus::event::emit;


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
    // r: vector<T>,
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