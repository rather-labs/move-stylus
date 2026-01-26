# Peep

The `peep` function enables cross-account storage reads. It allows you to inspect the fields of an object owned by a specific address, provided you know the object's unique identifier.

```move
module stylus::peep;

use stylus::object::UID;

public native fun peep<T: key>(owner_address: address, id: &UID): &T;
```

* **Immutable Access**: `peep` returns an immutable reference (`&T`). This strictly enforces read-only access; you can inspect the data, but you cannot modify the object or move it out of the owner's storage.

* **Type Safety**: The function is generic over type `T`. The caller must specify the exact struct type expected in storage. If the data at that location does not match the layout of `T`, we will get a runtime error.

## Implementation Example 

The following example demonstrates how one user can inspect another user's storage.

1.  **Creation**: Alice calls `create_foo` to generate a `Foo` struct. This object is moved into her storage namespace.
2.  **Observation**: Bob can then read Alice's `Foo` instance by calling `peep_foo`. He must provide Alice's address and the specific `UID` of the object he wishes to inspect.

```move
module test::peep;

use stylus::{
    peep as stylus_peep, 
    object::{Self, UID}, 
    tx_context::TxContext, 
    transfer::transfer
};

public struct Foo has key, store {
    id: UID,
    secret: u32
}

/// Alice calls this to create and own her Foo object.
entry fun create_foo(ctx: &mut TxContext) {
    let foo = Foo {
        id: object::new(ctx),
        secret: 42
    };

    transfer(foo, ctx.sender());
}

/// Bob (or anyone) calls this to read a Foo object 
/// owned by a Alice.
entry fun peep_foo(owner: address, foo_id: &UID): &Foo {
    stylus_peep::peep<Foo>(owner, foo_id)
}
```