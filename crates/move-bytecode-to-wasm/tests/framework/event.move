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

#[ext(event, indexes = 3)]
public struct TestEvent5 has copy, drop {
    a: u32,
    b: address,
    c: vector<u8>,
}

#[ext(event, indexes = 3)]
public struct TestEvent6 has copy, drop {
    a: u32,
    b: address,
    c: TestEvent2,
}

#[ext(event, indexes = 3)]
public struct TestEvent7 has copy, drop {
    a: u32,
    b: vector<u8>,
    c: TestEvent2,
}

#[ext(event, indexes = 1)]
public struct TestEvent8 has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[ext(event, indexes = 2)]
public struct TestEvent9 has copy, drop {
    a: u64,
    b: std::ascii::String,
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

public fun emit_test_event5(a: u32, b: address, c: vector<u8>) {
    emit(TestEvent5 { a, b, c });
}

public fun emit_test_event6(a: u32, b: address, c: TestEvent2) {
    emit(TestEvent6 { a, b, c });
}

public fun emit_test_event7(a: u32, b: vector<u8>, c: TestEvent2) {
    emit(TestEvent7 { a, b, c });
}

public fun emit_test_event8(a: u64, b: std::ascii::String) {
    emit(TestEvent8 { a, b });
}

public fun emit_test_event9(a: u64, b: std::ascii::String) {
    emit(TestEvent9 { a, b });
}

#[ext(event, anonymous, indexes = 1)]
public struct TestEvent1Anon has copy, drop {
    n: u32
}

#[ext(event, anonymous, indexes = 3)]
public struct TestEvent2Anon has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

#[ext(event, anonymous, indexes = 2)]
public struct TestEvent3Anon has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
}

#[ext(event, anonymous, indexes = 2)]
public struct TestEvent4Anon has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
    e: TestEvent2,
}

#[ext(event, anonymous, indexes = 3)]
public struct TestEvent5Anon has copy, drop {
    a: u32,
    b: address,
    c: vector<u8>,
}

#[ext(event, anonymous, indexes = 3)]
public struct TestEvent6Anon has copy, drop {
    a: u32,
    b: address,
    c: TestEvent2,
}

#[ext(event, anonymous, indexes = 3)]
public struct TestEvent7Anon has copy, drop {
    a: u32,
    b: vector<u8>,
    c: TestEvent2,
}

#[ext(event, anonymous, indexes = 1)]
public struct TestEvent8Anon has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[ext(event, anonymous, indexes = 2)]
public struct TestEvent9Anon has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[ext(event, anonymous, indexes = 4)]
public struct Anonymous has copy, drop {
    a: u32,
    b: u128,
    c: vector<u8>,
    d: TestEvent2,
}

#[ext(event, anonymous, indexes = 4)]
public struct Anonymous2 has copy, drop {
    a: u32,
    b: u128,
    c: vector<u8>,
    d: TestEvent2,
    e: u32,
    f: address,
    g: u128,
    h: vector<u8>,
    i: TestEvent2,
}

public fun emit_test_anon_event1(n: u32) {
    emit(TestEvent1Anon { n });
}

public fun emit_test_anon_event2(a: u32, b: address, c: u128) {
    emit(TestEvent2Anon { a, b, c });
}

public fun emit_test_anon_event3(a: u32, b: address, c: u128, d: vector<u8>) {
    emit(TestEvent3Anon { a, b, c, d });
}

public fun emit_test_anon_event4(a: u32, b: address, c: u128, d: vector<u8>, e: TestEvent2) {
    emit(TestEvent4Anon { a, b, c, d, e });
}

public fun emit_test_anon_event5(a: u32, b: address, c: vector<u8>) {
    emit(TestEvent5Anon { a, b, c });
}

public fun emit_test_anon_event6(a: u32, b: address, c: TestEvent2) {
    emit(TestEvent6Anon { a, b, c });
}

public fun emit_test_anon_event7(a: u32, b: vector<u8>, c: TestEvent2) {
    emit(TestEvent7Anon { a, b, c });
}

public fun emit_test_anon_event8(a: u64, b: std::ascii::String) {
    emit(TestEvent8Anon { a, b });
}

public fun emit_test_anon_event9(a: u64, b: std::ascii::String) {
    emit(TestEvent9Anon { a, b });
}

public fun emit_test_anonymous(a: u32, b: u128, c: vector<u8>, d: TestEvent2) {
    emit(Anonymous { a, b, c, d });
}

public fun emit_test_anonymous2(
    a: u32,
    b: u128,
    c: vector<u8>,
    d: TestEvent2,
    e: u32,
    f: address,
    g: u128,
    h: vector<u8>,
    i: TestEvent2,
) {
    emit(Anonymous2{ a, b, c, d, e, f, g, h, i });
}
