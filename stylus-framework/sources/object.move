module stylus::object;

use stylus::tx_context::TxContext;

/// References a object ID
public struct ID has copy, drop, store {
    bytes: address,
}

/// Globally unique IDs that define an object's ID in storage. Any object, that is a struct
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
public native fun delete(id: UID);

/// Named IDs are used know where the object will saved in storage, so we don't depend on the
/// user to pass the object UID to retrieve it from storage.
///
/// This struct is an special struct managed by the compiler. The name is given by the T struct
/// passed as type parameter. For example:
///
/// ```move
/// public struct TOTAL_SUPPLY has key {}
///
/// public struct TotalSupply has key {
///     id: NamedId<TOTAL_SUPPLY>,
///     total: u256,
/// }
/// ```
///
/// `NamedId`'s can only be used in one struct. Detecting the same NamedId in two different
/// structs will result in a compilation error.
public struct NamedId<phantom T: key> has store {
    id: ID,
}

native fun compute_named_id<T: key>(): address;

public fun new_named_id<T: key>(): NamedId<T> {
    NamedId {
        id: ID { bytes: compute_named_id<T>() },
    }
}
