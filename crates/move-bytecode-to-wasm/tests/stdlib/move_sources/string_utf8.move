module test::string_utf8;

use std::string::{String, utf8};

entry fun pack_utf8(string_bytes: vector<u8>): String {
    utf8(string_bytes)
}


entry fun pack_utf8_2(): (String, String) {
    (
        utf8(b"Привет мир"),
        utf8(b"こんにちは 世界"),
    )
}

entry fun pack_utf8_3(): (String, u16, String) {
    (
        utf8(b"hello world"),
        42,
        utf8(b"test string"),
    )
}

entry fun pack_utf8_4(): (String, vector<u16>, String) {
    (
        utf8(b"hello world"),
        vector[3, 1, 4, 1, 5],
        utf8(b"test string"),
    )
}

entry fun pack_unpack_utf8(value: String): String {
    value
}

entry fun pack_unpack_utf8_2(value: String, value_2: String): (String, String) {
    (value, value_2)
}

entry fun unpack_utf8(value: String): bool {
    value.as_bytes() == b"dlrow olleh"
}

entry fun unpack_utf8_2(value: String, value_2: String): bool {
    value.as_bytes() == b"hello world"
        && value_2.as_bytes() == b"test string"
}

entry fun unpack_utf8_3(value: String, n: u16, value_2: String): bool {
    value.as_bytes() == b"hello world"
        && n == 42
        && value_2.as_bytes() == b"test string"
}

entry fun unpack_utf8_4(value: String, n: vector<u16>, value_2: String): bool {
    value.as_bytes() == b"hello world"
        && n == vector[3, 1, 4, 1, 5]
        && value_2.as_bytes() == b"test string"
}

entry fun test_insert(mut s: String, at: u64, o: String): String {
    s.insert(at, o);
    s
}

entry fun test_custom_insert(s: &mut String, at: u64, o: String): String {
    let l = s.length();
    let mut front = s.substring(0, at);
    let end = s.substring(at, l);
    front.append(o);
    front.append(end);
    front
}

entry fun test_substring(mut s: String, i: u64, j: u64): String {
    s.substring(i, j)
}

entry fun test_append(mut s: String, o: String): String {
    s.append(o);
    s
}

entry fun test_partition_string(mut s: String, i: u64): (String, String) {
    let l = s.length();
    let front = s.substring(0, i);
    let end = s.substring(i, l);
    (front, end)
}