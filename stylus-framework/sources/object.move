module stylus::object;

use stylus::tx_context::TxContext;

public struct ID has copy, drop, store {
    bytes: address,
}

public struct UID has store {
    id: ID,
}

/// Create a new object. Returns the `UID` that must be stored in a Sui object.
/// This is the only way to create `UID`s.
public fun new(ctx: &mut TxContext): UID {
    UID {
        id: ID { bytes: ctx.fresh_object_address() },
    }
}

/// Delete the object and its `UID`. This is the only way to eliminate a `UID`.
/// This exists to inform Sui of object deletions. When an object
/// gets unpacked, the programmer will have to do something with its
/// `UID`. The implementation of this function emits a deleted
/// system event so Sui knows to process the object deletion
public native fun delete<T: key>(obj: T);
