module stylus::object;

use stylus::{
    tx_context::TxContext, 
    event::emit
};

/// Allows calling `.to_address` on a `UID` to get an `address`.
public use fun uid_to_address as UID.to_address;
public use fun uid_to_inner as UID.to_inner;
public use fun remove as NamedId.delete;

/// References a object ID
public struct ID has copy, drop, store {
    bytes: address,
}

/// Globally unique IDs that define an object's ID in storage. Any object, that is a struct
/// with the `key` ability, must have `id: UID` as its first field.
public struct UID has store {
    id: ID,
}

#[ext(event(indexes = 1))]
public struct NewUID has copy, drop {
    uid: ID,
}

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

/// Deletes the object from the storage.
public native fun delete(id: UID);

/// Generate a new UID specifically used for creating a UID from a hash
public(package) fun new_uid_from_hash(bytes: address): UID {
    UID { id: ID { bytes } }
}

/// Get the inner bytes of `id` as an address.
public fun uid_to_address(uid: &UID): address {
    uid.id.bytes
}

/// Get the raw bytes of a `uid`'s inner `ID`
public fun uid_to_inner(uid: &UID): ID {
    uid.id
}

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
    NamedId { id: ID { bytes: compute_named_id<T>() } }
}

/// Deletes the object with a `NamedId` from the storage.
public native fun remove<T: key>(id: NamedId<T>);

public(package) native fun as_uid<T: key>(named_id: &NamedId<T>): &UID;
public(package) native fun as_uid_mut<T: key>(named_id: &mut NamedId<T>): &mut UID;


/// Storage lookup structs
public struct FrozenStorageObject<T: key> {}
public struct OwnedStorageObject<T: key> {}
public struct SharedStorageObject<T: key> {}
