# String

Move does not support a built-in string type. Instead, it have two implementations in the standard library: The `std::string` module provides a UTF-8 encoded `String` type, while the `std::ascii` module provides an ASCII-only `String` type.

## Strings are bytes

In Move, strings are represented as sequences of bytes. The `std::string::String` type is a wrapper around a vector of bytes (`vector<u8>`), which allows it to store UTF-8 encoded text. The `std::ascii::String` type is similarly a wrapper around a vector of bytes, but it enforces that all bytes are valid ASCII characters (values from 0 to 127). Both modules provide functions for creating, manipulating, and querying strings and safety checks.

## Working with UTF-8 Strings

The `String` type in the `std::string` module is defined as follows:

```move
module std::string;

/// A `String` holds a sequence of bytes which is guaranteed to be in utf8 format.
public struct String has copy, drop, store {
    bytes: vector<u8>,
}
```

### Creating a UTF-8 String

You can create a UTF-8 string using the `utf8` function. It can also be created using the alias `.to_string()` on the `vector<u8>`.

```move
use std::string;

let utf8_str = string::utf8(b"Hello");

let another_utf8_str = b"Hello".to_string();
```

### Common Operations

UTF-8 strings in Move provide several methods for working with text. The most common operations include concatenation, slicing, and retrieving the length. For custom operations, the `bytes()` method can be used to access the underlying byte vector.

```move
let mut str = b"Hello,".to_string();
let another = b" World!".to_string();

// `append(String)` adds content to the end of the string
str.append(another);

// `sub_string(start, end)` copies a slice of the string
str.sub_string(0, 5); // "Hello"

// `length()` returns the number of bytes in the string
str.length(); // 12 (bytes)

// Methods can also be chained! For example, get the length of a substring
str.sub_string(0, 5).length(); // 5 (bytes)

// Check whether the string is empty
str.is_empty(); // false

// Access the underlying byte vector for custom operations
let bytes: &vector<u8> = str.bytes();
```

### Safe UTF-8 Operations

The default `utf8` method may abort if the provided bytes are not valid UTF-8. If you are unsure whether the bytes are valid, use the `try_utf8` method instead. This method returns an `Option<String>`:

- `Some(String)` if the bytes form a valid UTF-8 string.
- `None` if the bytes are invalid.

> [!NOTE]
> Functions with names starting with `try_*` typically return an `Option`. If the operation succeeds, the result is wrapped in `Some`. If it fails, the function returns `None`.


### UTF-8 Limitations

The `string` module does not provide a way to access individual characters directly. This is because UTF-8 is a variable-length encoding, where a character can occupy anywhere from 1 to 4 bytes. As a result, the `length()` method returns the number of **bytes** in the string, not the number of characters.

<!--
TODO: uncomment when native functions implemented
However, methods such as `sub_string` and `insert` validate character boundaries and will abort if the specified index falls within the middle of a character.
-->

## Working with ASCII Strings

The `String` type in the `std::ascii` module is defined as follows:

```move
module std::ascii;

/// A `String` holds a sequence of bytes which are guaranteed to be valid ASCII characters.
public struct String has copy, drop, store {
    bytes: vector<u8>,
}
```

### Creating an ASCII String

You can create an ASCII string using the `string` function. It can also be created using the alias `.to_string()` on a `std::string::String` type.

```move
use std::ascii::{Self};

let ascii_str = ascii::string(b"Hello");

// The first to_string() converts from byte array to std::string::String
// The second to_string() converts from std::string::String to std::ascii::String
let another_ascii_str = b"Hello".to_string().to_string();
```

### Common Operations

ASCII strings in Move provide similar methods to UTF-8 strings for working with text. The most common operations include concatenation, slicing, and retrieving the length.

```move
use std::ascii::{Self};

let mut str = ascii::string(b"Hello,");
let another = ascii::string(b" World!");

// `append(String)` adds content to the end of the string
str.append(another);

// `substring(start, end)` copies a slice of the string
str.substring(0, 5); // "Hello"

// `length()` returns the number of bytes in the string
str.length(); // 12 (bytes)

// Check whether the string is empty
str.is_empty(); // false

// Vector as bytes
let bytes: &vector<u8> = str.as_bytes();
```

### Safe ASCII Operations


Unlike UTF-8, ASCII strings are guaranteed to be single-byte characters. This means operations like `length()` directly return the number of characters, since each character is exactly one byte.

If you are unsure whether the bytes are valid ASCII, you can use the `try_string` method. It returns an `Option<String>`:

- **Some(String)**: if the bytes form a valid ASCII string.
- **None**: if the bytes are invalid.

