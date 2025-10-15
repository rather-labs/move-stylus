module erc20call::erc20call;

#[ext(external_call)]
public struct ERC20Call has drop {
    contract_address: address,
    delegate: bool
}

public fun new(contract_address: address, delegate: bool): ERC20Call {
    ERC20Call {
        contract_address,
        delegate,
    }
}

#[ext(external_call, view)]
public native fun total_supply(self: &ERC20Call): u256;

#[ext(external_call, view)]
public native fun balance_of(self: &ERC20Call, account: address): u256;

#[ext(external_call)]
public native fun transfer(self: &ERC20Call, account: address, amount: u256): bool;

#[ext(external_call, view)]
public native fun allowance(self: &ERC20Call, owner: address, spender: address): u256;

#[ext(external_call)]
public native fun approve(self: &ERC20Call, spender: address, amount: u256): bool;

#[ext(external_call)]
public native fun transfer_from(self: &ERC20Call, sender: address, recipient: address, amount: u256): bool;
