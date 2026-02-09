// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

/// This module extends the Dynamic Field functionality to work with Named IDs.
/// It provides a high-level API for adding, borrowing, and removing dynamic fields
/// from deterministic singleton objects (NamedId) by internally casting them
/// to their underlying UID representation.

module stylus::dynamic_field_named_id;

use stylus::{
    object::{Self, UID, NamedId},
    dynamic_field as dynamic_field
};
use std::option::Option;

/// Adds a dynamic field to a deterministic object identified by a `NamedId`.
///
/// Internally converts the `NamedId` reference to a `UID` to leverage
/// the standard dynamic_field logic.
public fun add<NId, Name: copy + drop + store, Value: store>(
    object: &mut NamedId<NId>,
    name: Name,
    value: Value,
) {
    let uid = object.as_uid_mut();
    dynamic_field::add(uid, name, value);
}

/// Immutably borrows a dynamic field from an object identified by a `NamedId`.
///
/// @abort EFieldDoesNotExist if the field is not found.
/// @abort EFieldTypeMismatch if the field exists but the value type is different.
public fun borrow<NId, Name: copy + drop + store, Value: store>(
    object: &NamedId<NId>,
    name: Name
): &Value {
    let uid = object.as_uid();
    dynamic_field::borrow(uid, name)
}

/// Mutably borrows a dynamic field from an object identified by a `NamedId`.
public fun borrow_mut<NId, Name: copy + drop + store, Value: store>(
    object: &mut NamedId<NId>,
    name: Name,
): &mut Value {
    let uid = object.as_uid_mut();
    dynamic_field::borrow_mut(uid, name)
}

/// Removes a dynamic field from a deterministic object and returns its value.
public fun remove<NId, Name: copy + drop + store, Value: store>(
    object: &mut NamedId<NId>,
    name: Name
): Value {
    let uid = object.as_uid_mut();
    dynamic_field::remove(uid, name)
}

/// Returns true if a dynamic field with the specified name exists on the Named object.
public fun exists_<NId, Name: copy + drop + store>(
    object: &NamedId<NId>,
    name: Name
): bool {
    let uid = object.as_uid();
    dynamic_field::exists_(uid, name)
}

/// Safely removes a dynamic field from a Named object if it exists.
///
/// Returns `Option::some(Value)` if found and removed, or `Option::none()` otherwise.
public fun remove_if_exists<NId, Name: copy + drop + store, Value: store>(
    object: &mut NamedId<NId>,
    name: Name,
): Option<Value> {
    let uid = object.as_uid_mut();
    dynamic_field::remove_if_exists(uid, name)
}

/// Returns true if a dynamic field exists and matches the specified `Value` type.
public fun exists_with_type<NId, Name: copy + drop + store, Value: store>(
    object: &NamedId<NId>,
    name: Name,
): bool {
    let uid = object.as_uid();
    dynamic_field::exists_with_type<Name, Value>(uid, name)
}
