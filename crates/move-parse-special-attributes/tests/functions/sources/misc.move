module test::misc;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID, NamedId}
};
use test::misc_external::{ExternalKeyStruct, new_external_key_struct};
use test::misc_external_2::{ExternalKeyStruct as ExternalKeyStruct2, new_external_key_struct as new_external_key_struct2};

public struct KeyStruct has key {
    id: UID,
    owner: address,
    value: u64
}

// return struct with key ability is invalid for entry functions
entry fun return_key_struct(value: u64, ctx: &mut TxContext): KeyStruct {
    return (KeyStruct { id: object::new(ctx), owner: ctx.sender(), value })
}

// return struct with key ability is valid for entry functions
entry fun return_ref_to_key_struct(value: u64, ctx: &mut TxContext): &KeyStruct {
    &KeyStruct { id: object::new(ctx), owner: ctx.sender(), value }
}

// return a external struct with key ability is invalid for entry functions
entry fun return_external_key_struct(value: u64, ctx: &mut TxContext): ExternalKeyStruct {
    new_external_key_struct(value, ctx)
}


// return a external struct with key ability is invalid for entry functions
entry fun return_external_key_struct_2(value: u64, ctx: &mut TxContext): ExternalKeyStruct2 {
    new_external_key_struct2(value, ctx)
}

// uid as argument is invalid
entry fun invalid_uid_argument(uid: UID): uid {
    uid
}

public struct NamedIdStruct  {}

// namedid as argument is invalid
entry fun invalid_namedid_argument(namedid: NamedId<NamedIdStruct>): NamedId<NamedIdStruct> {
    namedid
}

// ref uid as argument is valid
entry fun valid_ref_uid_argument(uid: &UID): &UID {
    uid
}