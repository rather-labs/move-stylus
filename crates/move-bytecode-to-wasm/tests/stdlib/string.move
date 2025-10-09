module test::string;

use std::ascii::String;

public fun pack_ascii(): String {
     b"hello world".to_ascii_string()
}

public fun pack_ascii_2(): (String, String) {
    (
        b"hello world".to_ascii_string(),
        b"test string".to_ascii_string(),
    )
}

public fun pack_ascii_3(): (String, u16, String) {
    (
        b"hello world".to_ascii_string(),
        42,
        b"test string".to_ascii_string(),
    )
}

public fun pack_ascii_4(): (String, vector<u16>, String) {
    (
        b"hello world".to_ascii_string(),
        vector[3, 1, 4, 1, 5],
        b"test string".to_ascii_string(),
    )
}

public fun pack_unpack_ascii(value: String): String {
    value
}

public fun pack_unpack_ascii_2(value: String, value_2: String): (String, String) {
    (value, value_2)
}

public fun unpack_ascii(value: String): bool {
    value.as_bytes() == b"dlrow olleh"
}

public fun unpack_ascii_2(value: String, value_2: String): bool {
    value.as_bytes() == b"hello world"
        && value_2.as_bytes() == b"test string"
}

public fun unpack_ascii_3(value: String, n: u16, value_2: String): bool {
    value.as_bytes() == b"hello world"
        && n == 42
        && value_2.as_bytes() == b"test string"
}

public fun unpack_ascii_4(value: String, n: vector<u16>, value_2: String): bool {
    value.as_bytes() == b"hello world"
        && n == vector[3, 1, 4, 1, 5]
        && value_2.as_bytes() == b"test string"
}
