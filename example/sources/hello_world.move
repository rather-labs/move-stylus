module hello_world::hello_world;

use std::ascii::String;

public fun unpack_ascii_2(value: String, value_2: String): bool {
    value.as_bytes() == b"hello world"
        && value_2.as_bytes() == b"test string"
}
