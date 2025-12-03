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

#[test]
fun test_increment_once() {
    let mut c = Counter { value: 0 };
    increment(&mut c);
    assert!(c.value == 1, debug::print(&c.value));
}

#[test]
fun test_increment_multiple_times() {
    let mut c = Counter { value: 10 };
    increment(&mut c);
    increment(&mut c);
    increment(&mut c);
    assert!(c.value == 13);
}
