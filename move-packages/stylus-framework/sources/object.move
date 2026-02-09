// Copyright (c) The Move Contributors
// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// This module defines the core identity system for the Stylus framework.
/// It implements unique identifiers (UIDs), deterministic Named IDs, 
/// and storage lookup structures that manage the lifecycle and addressable 
/// location of Move objects in global storage.

module stylus::object;

use stylus::{
    tx_context::TxContext, 
    event::emit
};

/// Method aliases for improved developer ergonomics.
public use fun uid_to_address as UID.to_address;
public use fun uid_to_inner as UID.to_inner;
public use fun remove as NamedId.delete;

/// A wrapper around a 32-byte identifier (address) representing an object's ID.
public struct ID has copy, drop, store {
    bytes: address,
}

/// A globally unique identifier that defines an object's identity in storage.
///
/// Any struct with the `key` ability MUST have `id: UID` as its first field.
/// This is a fundamental requirement of the framework's object model.
public struct UID has store {
    id: ID,
}

#[ext(event(indexes = 1))]
/// Event emitted whenever a new unique identifier is generated.
public struct NewUID has copy, drop {
    uid: ID,
}

/// Creates a new `UID` by requesting a fresh address from the transaction context.
/// 
/// Emits a `NewUID` event to allow off-chain indexers and callers to track 
/// the identity of the newly created object.
public fun new(ctx: &mut TxContext): UID {
    let res = UID { id: ID { bytes: ctx.fresh_object_address() } };
    emit(NewUID { uid: res.to_inner() });
    res
}

/// Removes an object identified by the given `UID` from the storage.
public native fun delete(id: UID);

/// Internal utility for generating a UID from a specific hash.
public(package) fun new_uid_from_hash(bytes: address): UID {
    UID { id: ID { bytes } }
}

/// Returns the underlying address representation of the `UID`.
public fun uid_to_address(uid: &UID): address {
    uid.id.bytes
}

/// Returns the inner `ID` wrapper of the `UID`.
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
/// public struct TOTAL_SUPPLY {}
///
/// public struct TotalSupply has key {
///     id: NamedId<TOTAL_SUPPLY>,
///     total: u256,
/// }
/// ```
///
/// `NamedId`'s can only be used in one struct. Detecting the same NamedId in two different
/// structs will result in a compilation error.
public struct NamedId<phantom T> has store {
    id: ID,
}

/// Deterministically computes the address for a Named ID based on type `T`.
native fun compute_named_id<T>(): address;

/// Generates a new `NamedId` for the specified type marker `T`.
public fun new_named_id<T>(): NamedId<T> {
    NamedId { id: ID { bytes: compute_named_id<T>() } }
}

/// Removes an object identified by a `NamedId` from global storage.
public native fun remove<T>(id: NamedId<T>);

/// Internal cast to treat a NamedId reference as a standard UID.
public(package) native fun as_uid<T>(named_id: &NamedId<T>): &UID;

/// Internal cast to treat a mutable NamedId reference as a mutable UID.
public(package) native fun as_uid_mut<T>(named_id: &mut NamedId<T>): &mut UID;

/// Get the `UID` for `obj`.
/// Cannot be made public as the access to `UID` for a given object must be privileged, and
/// restrictable in the object's module.
native fun borrow_uid<T: key>(obj: &T): &UID;

/// Get the underlying `ID` of `obj`
public fun id<T: key>(obj: &T): ID {
    borrow_uid(obj).id
}

/// Borrow the underlying `ID` of `obj`
public fun borrow_id<T: key>(obj: &T): &ID {
    &borrow_uid(obj).id
}

/// Get the inner bytes for the underlying `ID` of `obj`
public fun id_address<T: key>(obj: &T): address {
    borrow_uid(obj).id.bytes
}
