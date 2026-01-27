# UID and ID
The use of the `UID` type is required on all types that have the `key` ability. Here we go deeper into `UID` and its usage.

## Definition

The `UID` type is defined in the `stylus::object` module and is a wrapper around an `ID` which, in turn, wraps the `address` type. The UIDs are guaranteed to be unique, and can't be reused after the object was deleted.

```move
module stylus::object;

/// References a object ID
public struct ID has copy, drop, store {
    bytes: address,
}

/// Globally unique IDs that define an object's ID in storage.
/// Any object, that is a struct with the `key` ability, must have `id: UID` as its first field.
public struct UID has store {
    id: ID,
}
```

### Conversion methods

The framework provides these conversion methods to "peek" inside the UID and extract its underlying data:

```move
/// Get the inner bytes of `id` as an address.
public fun uid_to_address(uid: &UID): address {
    uid.id.bytes
}
public use fun uid_to_address as UID.to_address;

/// Get the raw bytes of a `uid`'s inner `ID`
public fun uid_to_inner(uid: &UID): ID {
    uid.id
}
public use fun uid_to_inner as UID.to_inner;
```

## Creating a new UID

To ensure every object has a unique identity within the contract's storage, the framework generates a new `UID` using a `Keccak256` hash of three specific parameters:

1. **Block Timestamp**: Ties the ID to the time of creation.
2. **Block Number**: Ties the ID to the specific blockchain height.
3. **Global Counter**: Increments with every call to ensure uniqueness for multiple objects created within the same block or transaction.

By combining these elements, the framework creates a collision-resistant address that is unique across the entire network.

The `object::tx_context` module provides methods to access the required transaction information. Specifically, `tx_context::fresh_object_address` handles the logic for hashing the data and producing the new address.

When `object::new` is called, it wraps this address into a `UID` and emits a `NewUID` event. This allows users and off-chain indexers to identify the exact address of the newly created object.


```move
module stylus::object;

use stylus::tx_context::TxContext;
use stylus::event::emit;

/// Creates a new `UID`, which must be stored in an object's `id` field.
/// This is the only way to create `UID`s.
///
/// Each time a new `UID` is created, an event is emitted on topic 0.
/// This allows the transaction caller to capture and persist it for later
/// reference to the object associated with that `UID`
public fun new(ctx: &mut TxContext): UID {
    let res = UID { id: ID { bytes: ctx.fresh_object_address() } };
    emit(NewUID { uid: res.to_inner() });
    res
}

/// Event emitted when a new UID is created.
#[ext(event(indexes = 1))]
public struct NewUID has copy, drop {
    uid: ID,
}
```

## UID Lifecycle

When deleting an object from storage, we must account for the abilities of its fields. Specifically, the `UID` struct lacks the **drop** ability; it cannot simply go out of scope, or the Move compiler will complain. It must be handled explicitly.

The `object::delete` function is designed for this purpose. After an object is "unpacked" (deconstructed), this function consumes the `UID` **by value**, moving it into the `object::delete` scope.

From an implementation standpoint, this function performs a critical role: it triggers the complete removal of the Object's data from the contract's storage, wiping the specific slots occupied by the Object.

```move
module stylus::object;

/// Deletes the UID and removes the associated object from storage.
public native fun delete(id: UID);
