module test::callee_contract_interface;

use stylus::contract_calls::{ContractCallEmptyResult, ContractCallResult, CrossContractCall};

#[ext(external_call)]
public struct ExampleContract has drop {
    configuration: CrossContractCall,
}

public fun new(configuration: CrossContractCall): ExampleContract {
    ExampleContract { configuration }
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
public native fun call_empty_res_1(self: &ExampleContract): ContractCallEmptyResult;

#[ext(external_call)]
public native fun call_empty_res_2(self: &ExampleContract, value: u64): ContractCallEmptyResult;

#[ext(external_call)]
public native fun call_empty_res_3(self: &ExampleContract, foo: Foo): ContractCallEmptyResult;

#[ext(external_call)]
public native fun call_empty_res_4(self: &ExampleContract, bar: Bar): ContractCallEmptyResult;

#[ext(external_call)]
public native fun call_empty_res_5(self: &ExampleContract, vec_1: vector<u8>): ContractCallEmptyResult;

#[ext(external_call, payable)]
public native fun call_empty_res_1_payable(self: &ExampleContract): ContractCallEmptyResult;

#[ext(external_call, payable)]
public native fun call_empty_res_2_payable(self: &ExampleContract, value: u64): ContractCallEmptyResult;

#[ext(external_call, payable)]
public native fun call_empty_res_3_payable(self: &ExampleContract, foo: Foo): ContractCallEmptyResult;

#[ext(external_call, payable)]
public native fun call_empty_res_4_payable(self: &ExampleContract, bar: Bar): ContractCallEmptyResult;

#[ext(external_call, payable)]
public native fun call_empty_res_5_payable(self: &ExampleContract, vec_1: vector<u8>): ContractCallEmptyResult;

#[ext(external_call, view)]
public native fun call_view_1(self: &ExampleContract): ContractCallResult<u64>;

#[ext(external_call, view)]
public native fun call_view_2(self: &ExampleContract): ContractCallResult<Foo>;

#[ext(external_call, view)]
public native fun call_view_3(self: &ExampleContract): ContractCallResult<Bar>;

#[ext(external_call, view)]
public native fun call_view_4(self: &ExampleContract): ContractCallResult<vector<u8>>;

#[ext(external_call, pure)]
public native fun call_pure_1(self: &ExampleContract): ContractCallResult<u64>;

#[ext(external_call, pure)]
public native fun call_pure_2(self: &ExampleContract): ContractCallResult<Foo>;

#[ext(external_call, pure)]
public native fun call_pure_3(self: &ExampleContract): ContractCallResult<Bar>;

#[ext(external_call, pure)]
public native fun call_pure_4(self: &ExampleContract): ContractCallResult<vector<u8>>;

#[ext(external_call, view, pure)]
public native fun call_view_pure_1(self: &ExampleContract): ContractCallResult<u64>;

#[ext(external_call, view, pure)]
public native fun call_view_pure_2(self: &ExampleContract): ContractCallResult<Foo>;

#[ext(external_call, view, pure)]
public native fun call_view_pure_3(self: &ExampleContract): ContractCallResult<Bar>;

#[ext(external_call, view, pure)]
public native fun call_view_pure_4(self: &ExampleContract): ContractCallResult<vector<u8>>;

#[ext(external_call)]
public native fun call_1(self: &ExampleContract): ContractCallResult<u64>;

#[ext(external_call)]
public native fun call_2(self: &ExampleContract): ContractCallResult<Foo>;

#[ext(external_call)]
public native fun call_3(self: &ExampleContract): ContractCallResult<Bar>;

#[ext(external_call)]
public native fun call_4(self: &ExampleContract): ContractCallResult<vector<u8>>;

#[ext(external_call, payable)]
public native fun call_1_payable(self: &ExampleContract): ContractCallResult<u64>;

#[ext(external_call, payable)]
public native fun call_2_payable(self: &ExampleContract): ContractCallResult<Foo>;

#[ext(external_call, payable)]
public native fun call_3_payable(self: &ExampleContract): ContractCallResult<Bar>;

#[ext(external_call, payable)]
public native fun call_4_payable(self: &ExampleContract): ContractCallResult<vector<u8>>;
