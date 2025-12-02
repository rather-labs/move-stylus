module test::event;

use stylus::event::emit;

public enum TestEnum has copy, drop {
    One,
    Two,
    Three,
}

#[allow(unused_field)]
public struct NestedStruct has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

#[allow(unused_field)]
public struct NestedStruct2 has copy, drop {
    a: u32,
    b: vector<u16>,
    c: std::ascii::String,
}

#[allow(unused_field)]
#[ext(event, indexes = 1)]
public struct TestEvent1 has copy, drop {
    n: u32
}

#[allow(unused_field)]
#[ext(event, indexes = 3)]
public struct TestEvent2 has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

#[allow(unused_field)]
#[ext(event, indexes = 2)]
public struct TestEvent3 has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
}

#[allow(unused_field)]
#[ext(event, indexes = 2)]
public struct TestEvent4 has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
    e: NestedStruct,
}

#[allow(unused_field)]
#[ext(event, indexes = 3)]
public struct TestEvent5 has copy, drop {
    a: u32,
    b: address,
    c: vector<u8>,
}

#[allow(unused_field)]
#[ext(event, indexes = 3)]
public struct TestEvent6 has copy, drop {
    a: u32,
    b: address,
    c: NestedStruct,
}

#[allow(unused_field)]
#[ext(event, indexes = 3)]
public struct TestEvent7 has copy, drop {
    a: u32,
    b: vector<u8>,
    c: NestedStruct,
}

#[allow(unused_field)]
#[ext(event, indexes = 1)]
public struct TestEvent8 has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[allow(unused_field)]
#[ext(event, indexes = 2)]
public struct TestEvent9 has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[allow(unused_field)]
#[ext(event, indexes = 3)]
public struct TestEvent10 has copy, drop {
    a: u32,
    b: address,
    c: vector<vector<u8>>,
}

#[allow(unused_field)]
#[ext(event, indexes = 3)]
public struct TestEvent11 has copy, drop {
    a: u32,
    b: address,
    c: NestedStruct2,
}

#[allow(unused_field)]
#[ext(event, indexes = 2)]
public struct TestEvent12 has copy, drop {
    a: u64,
    b: vector<std::ascii::String>,
}

#[allow(unused_field)]
#[ext(event, indexes = 2)]
public struct TestEvent13 has copy, drop {
    a: u64,
    b: vector<TestEnum>,
}

entry fun emit_test_event1(n: u32) {
    emit(TestEvent1 { n });
}

entry fun emit_test_event2(a: u32, b: address, c: u128) {
    emit(TestEvent2 { a, b, c });
}

entry fun emit_test_event3(a: u32, b: address, c: u128, d: vector<u8>) {
    emit(TestEvent3 { a, b, c, d });
}

entry fun emit_test_event4(a: u32, b: address, c: u128, d: vector<u8>, e: u32, f: address, g: u128) {
    let e = NestedStruct {a: e, b: f, c: g };
    emit(TestEvent4 { a, b, c, d, e });
}

entry fun emit_test_event5(a: u32, b: address, c: vector<u8>) {
    emit(TestEvent5 { a, b, c });
}

entry fun emit_test_event6(a: u32, b: address, c: u32, d: address, e: u128) {
    let c = NestedStruct {a: c, b: d, c: e };
    emit(TestEvent6 { a, b, c });
}

entry fun emit_test_event7(a: u32, b: vector<u8>, c: u32, d: address, e: u128) {
    let c = NestedStruct {a: c, b: d, c: e };
    emit(TestEvent7 { a, b, c });
}

entry fun emit_test_event8(a: u64, b: std::ascii::String) {
    emit(TestEvent8 { a, b });
}

entry fun emit_test_event9(a: u64, b: std::ascii::String) {
    emit(TestEvent9 { a, b });
}

entry fun emit_test_event10(a: u32, b: address, c: vector<vector<u8>>) {
    emit(TestEvent10 { a, b, c });
}

entry fun emit_test_event11(a: u32, b: address, c: u32, d: vector<u16>, e: std::ascii::String) {
    let c = NestedStruct2 {a: c, b: d, c: e };
    emit(TestEvent11 { a, b, c });
}

entry fun emit_test_event12(a: u64, b: vector<std::ascii::String>) {
    emit(TestEvent12 { a, b });
}

entry fun emit_test_event13(a: u64, b: vector<TestEnum>) {
    emit(TestEvent13 { a, b });
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 1)]
public struct TestEvent1Anon has copy, drop {
    n: u32
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 3)]
public struct TestEvent2Anon has copy, drop {
    a: u32,
    b: address,
    c: u128,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 2)]
public struct TestEvent3Anon has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 2)]
public struct TestEvent4Anon has copy, drop {
    a: u32,
    b: address,
    c: u128,
    d: vector<u8>,
    e: NestedStruct,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 3)]
public struct TestEvent5Anon has copy, drop {
    a: u32,
    b: address,
    c: vector<u8>,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 3)]
public struct TestEvent6Anon has copy, drop {
    a: u32,
    b: address,
    c: NestedStruct,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 3)]
public struct TestEvent7Anon has copy, drop {
    a: u32,
    b: vector<u8>,
    c: NestedStruct,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 1)]
public struct TestEvent8Anon has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 2)]
public struct TestEvent9Anon has copy, drop {
    a: u64,
    b: std::ascii::String,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 3)]
public struct TestEvent10Anon has copy, drop {
    a: u32,
    b: address,
    c: vector<vector<u8>>,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 3)]
public struct TestEvent11Anon has copy, drop {
    a: u32,
    b: address,
    c: NestedStruct2,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 2)]
public struct TestEvent12Anon has copy, drop {
    a: u64,
    b: vector<std::ascii::String>,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 2)]
public struct TestEvent13Anon has copy, drop {
    a: u64,
    b: vector<TestEnum>,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 4)]
public struct Anonymous has copy, drop {
    a: u32,
    b: u128,
    c: vector<u8>,
    d: NestedStruct,
}

#[allow(unused_field)]
#[ext(event, anonymous, indexes = 4)]
public struct Anonymous2 has copy, drop {
    a: u32,
    b: u128,
    c: vector<u8>,
    d: NestedStruct,
    e: u32,
    f: address,
    g: u128,
    h: vector<u8>,
    i: NestedStruct,
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

entry fun emit_test_anon_event6(a: u32, b: address, c: u32, d: address, e: u128) {
    let c = NestedStruct {a: c, b: d, c: e };
    emit(TestEvent6Anon { a, b, c });
}

entry fun emit_test_anon_event7(a: u32, b: vector<u8>, c: u32, d: address, e: u128) {
    let c = NestedStruct {a: c, b: d, c: e };
    emit(TestEvent7Anon { a, b, c });
}

entry fun emit_test_anon_event8(a: u64, b: std::ascii::String) {
    emit(TestEvent8Anon { a, b });
}

entry fun emit_test_anon_event9(a: u64, b: std::ascii::String) {
    emit(TestEvent9Anon { a, b });
}

entry fun emit_test_anon_event10(a: u32, b: address, c: vector<vector<u8>>) {
    emit(TestEvent10Anon { a, b, c });
}

entry fun emit_test_anon_event11(a: u32, b: address, c: u32, d: vector<u16>, e: std::ascii::String) {
    let c = NestedStruct2 {a: c, b: d, c: e };
    emit(TestEvent11Anon { a, b, c });
}

entry fun emit_test_anon_event12(a: u64, b: vector<std::ascii::String>) {
    emit(TestEvent12Anon { a, b });
}

entry fun emit_test_anon_event13(a: u64, b: vector<TestEnum>) {
    emit(TestEvent13Anon { a, b });
}

entry fun emit_test_anonymous(a: u32, b: u128, c: vector<u8>, d: u32, e: address, f: u128) {
    let d = NestedStruct {a: d, b: e, c: f };
    emit(Anonymous { a, b, c, d });
}
