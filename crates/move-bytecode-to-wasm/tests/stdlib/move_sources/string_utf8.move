module test::string_utf8;

use std::string::{String, utf8};

entry fun pack_utf8(): String {
     // utf8(b"hello world")

 //       utf8(b"Привет мир")
   // utf8(b"こんにちは 世界")
    utf8(b"🐱")
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
