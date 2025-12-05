module test::misc;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::{UID, NamedId};
use test::misc_external::{ExternalKeyStruct, new_external_key_struct};

public struct KeyStruct has key {
    id: UID,
    owner: address,
    value: u64
}

// return struct with key ability is invalid for entry functions
entry fun return_key_struct(value: u64, ctx: &mut TxContext): KeyStruct {
    return (KeyStruct { id: object::new(ctx), owner: ctx.sender(), value })
}

// return struct with key ability is invalid for entry functions
entry fun return_ref_to_key_struct(value: u64, ctx: &mut TxContext): &KeyStruct {
    &KeyStruct { id: object::new(ctx), owner: ctx.sender(), value }
}

// TODO: right now we are not checking the external structs so we are not catching this error
entry fun return_external_key_struct(value: u64, ctx: &mut TxContext): ExternalKeyStruct {
    new_external_key_struct(value, ctx)
}

// uid as argument is invalid
entry fun invalid_uid_argument(uid: UID): uid {
    uid
}

public struct NamedIdStruct has key {}

// namedid as argument is invalid
entry fun invalid_namedid_argument(namedid: NamedId<NamedIdStruct>): NamedId<NamedIdStruct> {
    namedid
}

// ref uid as argument is valid
entry fun valid_ref_uid_argument(uid: &UID): &UID {
    uid
}