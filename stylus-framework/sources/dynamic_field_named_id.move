// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_const)]
module stylus::dynamic_field_named_id;

use stylus::{
    object::{Self, UID, NamedId}, 
    dynamic_field as dynamic_field
};


public fun add<NId: key, Name: copy + drop + store, Value: store>(
    object: &mut NamedId<NId>,
    name: Name,
    value: Value,
) {
    let uid = object.as_uid_mut();
    dynamic_field::add(uid, name, value);
}

public fun borrow<NId: key, Name: copy + drop + store, Value: store>(
    object: &NamedId<NId>,
    name: Name
): &Value {
    let uid = object.as_uid();
    dynamic_field::borrow(uid, name)
}

public fun borrow_mut<NId: key, Name: copy + drop + store, Value: store>(
    object: &mut NamedId<NId>,
    name: Name,
): &mut Value {
    let uid = object.as_uid_mut();
    dynamic_field::borrow_mut(uid, name)
}

public fun remove<NId: key, Name: copy + drop + store, Value: store>(
    object: &mut NamedId<NId>,
    name: Name
): Value {
    let uid = object.as_uid_mut();
    dynamic_field::remove(uid, name)
}

public fun exists_<NId: key, Name: copy + drop + store>(
    object: &NamedId<NId>,
    name: Name
): bool {
    let uid = object.as_uid();
    dynamic_field::exists_(uid, name)
}

public fun remove_if_exists<NId: key, Name: copy + drop + store, Value: store>(
    object: &mut NamedId<NId>,
    name: Name,
): Option<Value> {
    let uid = object.as_uid_mut();
    dynamic_field::remove_if_exists(uid, name)
}

public fun exists_with_type<NId: key, Name: copy + drop + store, Value: store>(
    object: &NamedId<NId>,
    name: Name,
): bool {
    let uid = object.as_uid();
    dynamic_field::exists_with_type<Name, Value>(uid, name)
}
