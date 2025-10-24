module test::callee_contract_interface;

use stylus::contract_calls::{ContractCallEmptyResult, ContractCallResult};

#[ext(external_call)]
public struct CrossContractCall has drop {
    contract_address: address,
    delegate: bool,
}

public fun new(contract_address: address, delegate: bool): CrossContractCall {
    CrossContractCall {
        contract_address,
        delegate,
    }
}

// Static abi sub-struct
#[allow(unused_field)]
public struct Baz has drop {
    a: u16,
    b: u128,
}

// Dynamic abi substruct
#[allow(unused_field)]
public struct Bazz has drop {
    a: u16,
    b: vector<u256>,
}

// Static abi struct
#[allow(unused_field)]
public struct Foo has drop {
    q: address,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
    baz: Baz,
}

// Dynamic abi struct
#[allow(unused_field)]
public struct Bar has drop {
    q: address,
    r: vector<u32>,
    s: vector<u128>,
    t: bool,
    u: u8,
    v: u16,
    w: u32,
    x: u64,
    y: u128,
    z: u256,
    bazz: Bazz,
    baz: Baz,
}

#[ext(external_call)]
public native fun call_empty_res_1(self: &CrossContractCall): ContractCallEmptyResult;

#[ext(external_call)]
public native fun call_empty_res_2(self: &CrossContractCall, value: u64): ContractCallEmptyResult;

#[ext(external_call)]
public native fun call_empty_res_3(self: &CrossContractCall, foo: Foo): ContractCallEmptyResult;

#[ext(external_call)]
public native fun call_empty_res_4(self: &CrossContractCall, bar: Bar): ContractCallEmptyResult;

#[ext(external_call)]
public native fun call_empty_res_5(self: &CrossContractCall, vec_1: vector<u8>): ContractCallEmptyResult;

#[ext(external_call, view)]
public native fun call_view_1(self: &CrossContractCall): ContractCallResult<u64>;
