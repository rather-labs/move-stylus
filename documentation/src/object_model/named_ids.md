# Named Ids

While standard `UID`s are generated dynamically, the framework also supports `NamedId`s. These are used when the storage location of an object must be deterministic and predictable. This allows the system or other contracts to retrieve an object without requiring the user to manually provide its `UID` during every transaction.

## Definition

A `NamedId` is a specialized struct managed by the compiler. It utilizes a **phantom** type parameter `T` to derive a deterministic address. This ensures that a specific type always maps to the same coordinates in storage. To maintain safety and compatibility with the storage model, the generic type `T` must be a struct with the `key` ability.

```move
module stylus::object;

/// Named IDs provide a deterministic storage location based on type `T`.
/// Each `NamedId` can only be associated with one struct definition.
public struct NamedId<phantom T: key> has store {
    id: ID,
}
```

## Deriving a `NamedId`

The `stylus::object::new_named_id` function is responsible for creating a new `NamedId` for a given generic type `T`. Intenally, the native function `compute_named_id` performs a `Keccak256` hash of the generic struct's fully qualified name.

```move
native fun compute_named_id<T: key>(): address;

public fun new_named_id<T: key>(): NamedId<T> {
    NamedId { id: ID { bytes: compute_named_id<T>() } }
}
```

### Example

The following snippet demonstrates how to implement a Singleton pattern using `NamedId`s. We define a `COUNTER_` struct to derive the id for our `Counter` object.

```move
module example::counter;

use stylus::{
    tx_context::TxContext,
    object::{Self, NamedId},
    transfer::{Self}
};

// The struct which name will be used to generate the NamedId
public struct COUNTER_ has key {}

// A storage object with a NamedId as first field.
public struct Counter has key {
    id: NamedId<COUNTER_>,
    value: u64
}

// To create the counter, we call the object::new_named_id function over type COUNTER_ to get the NamedId.
entry fun create(ctx: &TxContext) {
  transfer::share_object(Counter {
    id: object::new_named_id<COUNTER_>(),
    value: 42
  });
}
```

## `NamedId` Lifecycle

In the same fashion we handled the deletion of Objects with `UID`, the framework provides a native function to delete Objects with `NamedId`.

```move
/// Deletes the object with a `NamedId` from the storage.
public native fun remove<T: key>(id: NamedId<T>);

```

> [!NOTE]
> `NamedId`s remove function is re-exported as `delete`, so it can be used the way as `UID`'s `delete` function.
