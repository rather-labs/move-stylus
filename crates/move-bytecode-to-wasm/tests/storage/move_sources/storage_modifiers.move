//! This module tests function storage modifiers.
module test::storage_modifiers;

use stylus::{transfer::{Self}, object::{Self, UID}, tx_context::TxContext};

public struct Object has key { id: UID, value: u64 }

entry fun create_owned_object(ctx: &mut TxContext) {
    let object = Object {
        id: object::new(ctx),
        value: 42,
    };
    transfer::transfer(object, ctx.sender());
}

entry fun create_shared_object(ctx: &mut TxContext) {
    let object = Object {
        id: object::new(ctx),
        value: 43,
    };
    transfer::share_object(object);
}

entry fun create_frozen_object(ctx: &mut TxContext) {
    let object = Object {
        id: object::new(ctx),
        value: 44,
    };
    transfer::freeze_object(object);
}

#[ext(owned_objects(o))]
entry fun locate_owned_object_fn(o: &Object): u64 {
    o.value
}

#[ext(shared_objects(o))]
entry fun locate_shared_object_fn(o: &mut Object): u64 {
    o.value
}

#[ext(frozen_objects(o))]
entry fun locate_frozen_object_fn(o: &Object): u64 {
    o.value
}

entry fun locate_object_no_modifier_fn(o: &Object): u64 {
    o.value
}

#[ext(owned_objects(a), shared_objects(b), frozen_objects(c))]
entry fun locate_many_objects_fn(a: &Object, b: &Object, c: &Object): u64 {
    a.value + b.value + c.value
}

#[ext(shared_objects(b), frozen_objects(c))]
entry fun locate_many_objects_2_fn(a: &Object, b: &Object, c: &Object): u64 {
    a.value + b.value + c.value
}