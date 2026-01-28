# Ability store

The `key` ability requires all fields to have `store`, which defines what the `store` ability means: it is the ability to serve as a field of an Object. A struct with `copy` or `drop` but without `store` can never be stored. A type with `key` but without `store` cannot be _wrapped_ - used as a field—in another object, and is constrained to always remain at the top level.

## Definition

The `store` ability allows a type to be used as a field in a struct with the `key` ability.

```move
use std::string::String;

/// Extra metadata with `store`; all fields must have `store` as well!
public struct Metadata has store {
    bio: String,
}

/// An object for a single user record.
public struct User has key {
    id: UID,
    name: String,       // String has `store`
    age: u8,            // All integers have `store`
    metadata: Metadata, // Another type with the `store` ability
}
```

>[!Note]
All native types (except references) in Move have the `store` ability. All of the types defined in the standard library have the `store` ability as well.