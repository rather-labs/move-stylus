module hello_world::erc20;

// use std::string::String;
use stylus::tx_context::TxContext;
use stylus::object::{compute_id, UID};
use stylus::transfer::share_object;
use stylus::table::{Table, contains};
use stylus::table as table;
use stylus::storage::{read_shared, read_shared_mut};

public struct Erc20 has key {
    id: UID,
    balances:   Table<address, u64>,                     // mapping(address => uint)
    allowances: Table<address, Table<address, u64>>,     // mapping(address => mapping(address => uint))
    total_supply: u64,
    name: vector<u8>,
    symbol: vector<u8>,
    decimals: u8,
}

fun init(ctx: &mut TxContext) {
    share_object(Erc20 {
        id: compute_id(b"erc_20_contract"),
        balances: table::new(ctx),
        allowances: table::new(ctx),
        total_supply: 0,
        name: b"Test Coin",
        symbol: b"TST",
        decimals: 18,
    });
}

fun get_contract(): &Erc20 {
    let id = compute_id(b"erc_20_contract");
    read_shared(id)
}

fun get_contract_mut(): &mut Erc20 {
    let id = compute_id(b"erc_20_contract");
    read_shared_mut(id)
}

public fun name(): &vector<u8> {
    let erc_20 = get_contract();
    &erc_20.name
}

public fun symbol(): &vector<u8> {
    let erc_20 = get_contract();
    &erc_20.symbol
}

public fun decimals(): u8 {
    let erc_20 = get_contract();
    erc_20.decimals
}

/// balances[addr] = value (create if missing)
fun set_balance(addr: address, value: u64) {
    let e = get_contract_mut();
    if (e.balances.contains(addr)) {
        let address_balance = e.balances.borrow_mut(addr);
        value;
    } else {
        e.balances.add(addr, value);
    }
}

public fun allowance(owner: address, spender: address): u64 {
    let e = get_contract();
    if (!e.allowances.contains(owner)) return 0;
    let inner = e.allowances.borrow(owner);
    if (inner.contains(spender)) { *inner.borrow(spender) } else { 0 }
}

public fun balance_of(owner: address): u64 {
    let e = get_contract();
    if (e.balances.contains(owner)) {
        *e.balances.borrow(owner)
    } else { 0 }
}

/*
public entry fun approve(owner: address, spender: address, amount: u64, ctx: &mut TxContext) {
    let e = get_contract_mut();

    if (!e.allowances.contains(owner)) {
        e.allowances.add(owner, table::new<address, u64>(ctx));
        // ^ if you keep ctx elsewhere, pass it instead. This illustrates "create inner table".
    };

    let owner = e.allowances.borrow_mut(owner);

    if (owner.contains(spender)) {
        let mut spender = owner.borrow_mut(spender);
        *spender = amount;
    } else {
        allowances(owner, spender, amount);
    }
}*/

public entry fun mint(to: address, amount: u64) {
    let e = get_contract_mut();
    let bal = balance_of(to);
    set_balance(to, bal + amount);
    e.total_supply = e.total_supply + amount;
}

public entry fun burn(from: address, amount: u64 /* add access control as needed */) {
    let e = get_contract_mut();
    let bal = balance_of(from);
    // assert!(bal >= amount, E_INSUFFICIENT_BALANCE);
    set_balance(from, bal - amount);
    e.total_supply = e.total_supply - amount;
}
