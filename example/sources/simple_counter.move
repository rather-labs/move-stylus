// Copyright (c) 2025 Rather Labs
// SPDX-License-Identifier: BUSL-1.1

module hello_world::simple_counter;

public struct Counter has drop {
    value: u64
}

/// Increment a counter by 1.
entry fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}


/// Read counter.
entry fun read(counter: &Counter): u64 {
    counter.value
}

/// Set value (only runnable by the Counter owner)
entry fun set_value(counter: &mut Counter, value: u64) {
    counter.value = value;
}

//
// Unit tests
//
#[test]
fun test_increment_multiple_times() {
    let mut c = Counter { value: 10 };
    increment(&mut c);
    increment(&mut c);
    increment(&mut c);
    assert!(c.value == 13);
}

#[test]
fun test_read_value() {
    let c = Counter { value: 42 };
    let v = read(&c);
    assert!(v == 42);
}

#[test]
fun test_set_value() {
    let mut c = Counter { value: 5 };
    set_value(&mut c, 99);
    assert!(c.value == 99);
}

#[test]
fun test_set_then_increment() {
    let mut c = Counter { value: 5 };
    set_value(&mut c, 20);
    increment(&mut c);
    assert!(c.value == 21);
}

#[test]
fun test_increment_once() {
    let mut c = Counter { value: 0 };
    increment(&mut c);
    assert!(c.value == 1);
}

#[test, expected_failure]
fun test_increment_once_fails() {
    let mut c = Counter { value: 0 };
    increment(&mut c);
    assert!(c.value == 2);
}
