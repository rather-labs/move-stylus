module stylus::object;

use stylus::tx_context::TxContext;

public struct ID has copy, drop, store {
    bytes: address,
}

/// Globally unique IDs that define an object's ID in storage. Any Sui Object, that is a struct
/// with the `key` ability, must have `id: UID` as its first field.
public struct UID has store {
    id: ID,
}

/// Creates a new `UID`, which must be stored in an object's `id` field.
/// This is the only way to create `UID`s.
///
/// Each time a new `UID` is created, an event is emitted on topic 0.
/// This allows the transaction caller to capture and persist it for later
/// reference to the object associated with that `UID`
public fun new(ctx: &mut TxContext): UID {
    UID {
        id: ID { bytes: ctx.fresh_object_address() },
    }
}

/// Deletes the object from the storage.
public native fun delete<T: key>(obj: T);
