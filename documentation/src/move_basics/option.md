# Option

The `Option` type in Move is a powerful way to represent values that may or may not be present. It is similar to the concept of nullable types in other programming languages but provides a more explicit and type-safe way to handle optional values. `Option` is defined in the `std::option` module of the [Standard Library](./standard_library.md). as follows:

```move
module std::option;

/// Abstraction of a value that may or may not be present.
public struct Option<Element> has copy, drop, store {
    vec: vector<Element>
}
```

> **Note**: The 'std::option' module is implicitly imported in every module, so you don't need to add an explicit import.

The `Option` type is a generic type parameterized by `Element`. It defines a single field, `vec`, which is a vector of `Element`. This vector can have a length of 0 or 1, representing the absence or presence of a value.

> **Note:** Although `Option` is implemented as a struct containing a vector rather than an enum, this design exists for historical reasons—`Option` was introduced before Move supported enums.

The `Option` type has two variants:
- **Some**: holds a value.
- **None**: indicates no value.

`Option` provides a type-safe way to represent the absence of a value, eliminating the need for empty or undefined values.

## In Practice

To illustrate why the `Option` type is useful, consider an application that collects user input and stores it in variables. Some fields are mandatory, while others are optional. For instance, a contact's email address is optional. Using an empty string to represent the absence of an email address would require additional checks to distinguish between an empty string and a missing value. Instead, the `Option` type can be used to represent the email directly.


```move
module book::contact_registry;

use std::string::String;

/// A struct representing a contact record.
public struct Contact has drop {
    name: String,
    email: Option<String>,
    phone: String,
}

/// Create a new `Contact` struct with the given fields.
public fun register(
    name: String,
    email: Option<String>,
    phone: String,
): Contact {
    Contact { name, email, phone }
}
```

In the previous example, the `email` field is defined as `Option<String>`. This means it can either hold a `String` value wrapped in `Some`, or be explicitly empty with `None`. By using `Option`, the optional nature of the field is made explicit, removing ambiguity and avoiding the need for extra checks to distinguish between an empty string and a missing value.

# Creating and Using Option values

To create an `Option` value, you can use the `some` and `none` constructors provided by the `std::option` module.

```move
// Creates an Option<u64> with a value of 42
let mut opt = option::some(42);
assert!(opt.is_some());

// Creates an empty Option<u64>
let empty: Option<u64> = option::none();
assert!(empty.is_none());

let mut opt_2 = option::some(b"Alice");

// internal value can be `borrow`ed and `borrow_mut`ed.
assert_ref_eq!(opt_2.borrow(), &b"Alice");

// `option.extract` takes the value out of the option, leaving the option empty.
let inner = opt_2.extract();

// `option.is_none()` returns true if option is None.
assert_eq!(opt.is_none(), true);
```
