// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

module erc20call::erc20call;

use stylus::contract_calls::{ContractCallResult, CrossContractCall};

#[ext(external_call)]
public struct ERC20(CrossContractCall) has drop;

public fun new(configuration: CrossContractCall): ERC20 {
    ERC20(configuration)
}

#[ext(external_call(view))]
public native fun total_supply(self: &ERC20): ContractCallResult<u256>;

#[ext(external_call(view))]
public native fun balance_of(self: &ERC20, account: address): ContractCallResult<u256>;

#[ext(external_call)]
public native fun transfer(self: &ERC20, account: address, amount: u256): ContractCallResult<bool>;

#[ext(external_call(view))]
public native fun allowance(self: &ERC20, owner: address, spender: address): ContractCallResult<u256>;

#[ext(external_call)]
public native fun approve(self: &ERC20, spender: address, amount: u256): ContractCallResult<bool>;

#[ext(external_call)]
public native fun transfer_from(self: &ERC20, sender: address, recipient: address, amount: u256): ContractCallResult<bool>;

