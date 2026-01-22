module test::struct_validations;

use stylus::tx_context::TxContext;
use stylus::object::{Self};
use stylus::object::{UID, NamedId};

public struct StructWithKeyNoUid has key {
    owner: address,
    value: u64
}

public struct StructWithKeyWrongUidName has key {
    bad: UID,
    owner: address,
    value: u64
}

public struct StructWithKeyManyUids has key {
    id: UID,
    id2: UID,
    owner: address,
    value: u64
}

public struct NamedIdStruct has key {}

public struct StructWithKeyManyNamedIds has key {
    id: NamedId<NamedIdStruct>,
    id2: NamedId<NamedIdStruct>,
    owner: address,
    value: u64
}

public struct StructWithoutKeyHasUidField {
    id: UID,
    owner: address,
    value: u64
}

#[ext(event(indexes = 1))]
public struct NestedEvent {
    data: vector<u8>,
}

#[ext(abi_error)]
public struct NestedError {
    data: vector<u8>,
}

public struct StructWithNestedEvent {
    event: NestedEvent,
}

public struct StructWithNestedError {
    error: NestedError,
}

public struct CrossContractCall {}
public struct ContractCallResult {}
public struct ContractCallEmptyResult {}
public struct Field {}
public struct Table {}
public struct ID {}
public struct UID {}
public struct NamedId {}