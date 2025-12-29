module hello_world::string_utf8;

use std::string::{String, utf8};

entry fun pack_utf8(): String {
     utf8(b"hello world")
}
