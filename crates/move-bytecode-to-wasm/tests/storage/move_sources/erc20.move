module test::erc20;

use stylus::{
    event::emit, 
    tx_context::TxContext, 
    transfer::{Self}, 
    object::{Self, NamedId}, 
    dynamic_field_named_id as field, 
    table::{Self, Table}
};
use std::ascii::{Self, String};

const EInssuficientFunds: u64 = 1;
const ENotAllowed: u64 = 2;

public struct TOTAL_SUPPLY  {}
public struct CONTRACT_INFO  {}
public struct ALLOWANCE_  {}
public struct BALANCE_  {}

public struct TotalSupply has key {
    id: NamedId<TOTAL_SUPPLY>,
    total: u256,
}

public struct Info has key {
    id: NamedId<CONTRACT_INFO>,
    name: String,
    symbol: String,
    decimals: u8,
}

#[ext(event, indexes = 2)]
public struct Transfer has copy, drop {
    from: address,
    to: address,
    value: u256
}

#[ext(event, indexes = 2)]
public struct Approval has copy, drop {
    owner: address,
    spender: address,
    value: u256
}

public struct Balance has key {
    id: NamedId<BALANCE_>,
}

public struct Allowance has key {
    id: NamedId<ALLOWANCE_>,
}

entry fun create() {
    transfer::freeze_object(Info {
        id: object::new_named_id<CONTRACT_INFO>(),
        name: ascii::string(b"Test Coin"),
        symbol: ascii::string(b"TST"),
        decimals: 18,
    });

    transfer::share_object(TotalSupply {
        id: object::new_named_id<TOTAL_SUPPLY>(),
        total: 0,
    });

    transfer::share_object(Allowance {
        id: object::new_named_id<ALLOWANCE_>(),
    });

    transfer::share_object(Balance {
        id: object::new_named_id<BALANCE_>(),
    });
}

entry fun mint(
    to: address,
    amount: u256,
    total_supply: &mut TotalSupply,
    balance: &mut Balance
) {
    if (field::exists_(&balance.id, to)) {
        let balance_amount = field::borrow_mut(&mut balance.id, to);
        *balance_amount = *balance_amount + amount;
    } else {
        field::add(&mut balance.id, to, amount);
    };

    total_supply.total = total_supply.total + amount;

    emit(Transfer {
        from: @0x0,
        to,
        value: amount
    });
}

entry fun burn(
    from: address,
    amount: u256,
    total_supply: &mut TotalSupply,
    balance: &mut Balance
) {
    if (amount > 0 && !field::exists_(&balance.id, from)) {
        abort(EInssuficientFunds)
    };

    let balance_amount = field::borrow_mut(&mut balance.id, from);

    if (*balance_amount < amount) {
        abort(EInssuficientFunds)
    };

    if (amount > total_supply.total) {
        *balance_amount = 0;
        total_supply.total = 0;
    } else {
        *balance_amount = *balance_amount - amount;
        total_supply.total = total_supply.total - amount;
    };

    emit(Transfer {
        from,
        to: @0x0,
        value: amount
    });
}

entry fun total_supply(t_supply: &TotalSupply): u256 {
    t_supply.total
}

entry fun decimals(contract_info: &Info): u8 {
    contract_info.decimals
}

entry fun name(contract_info: &Info): String {
    contract_info.name
}

entry fun symbol(contract_info: &Info): String {
    contract_info.symbol
}

entry fun balance_of(account: address, balance: &Balance): u256 {
    if (field::exists_(&balance.id, account)) {
        *field::borrow<BALANCE_, address, u256>(&balance.id, account)
    } else {
        0
    }
}

entry fun transfer(
    recipient: address,
    amount: u256,
    tx_context: &TxContext,
    balance: &mut Balance,
): bool {
    let sender_balance = field::borrow_mut<BALANCE_, address, u256>(
        &mut balance.id,
        tx_context.sender()
    );
    if (*sender_balance < amount) {
        abort(EInssuficientFunds)
    };

    *sender_balance = *sender_balance - amount;

    if (field::exists_(&balance.id, recipient)) {
        let recipient_balance = field::borrow_mut(&mut balance.id, recipient);
        *recipient_balance = *recipient_balance + amount;
    } else {
        field::add(&mut balance.id, recipient, amount);
    };

    emit(Transfer {
        from: tx_context.sender(),
        to: recipient,
        value: amount
    });

    true
}

entry fun approve(
    spender: address,
    amount: u256,
    allowance: &mut Allowance,
    ctx: &mut TxContext,
): bool {
    let spender_allowance = if (field::exists_(&allowance.id, ctx.sender())) {
        field::borrow_mut<ALLOWANCE_, address, Table<address, u256>>(&mut allowance.id, ctx.sender())
    } else {
        field::add(
            &mut allowance.id,
            ctx.sender(),
            table::new<address, u256>(ctx)
        );
        field::borrow_mut<ALLOWANCE_, address, Table<address, u256>>(&mut allowance.id, ctx.sender())
    };

    if (spender_allowance.contains(spender)) {
        let allowance = spender_allowance.borrow_mut(spender);
        *allowance = amount;
    } else {
        spender_allowance.add(spender, amount);
    };

    emit(Approval {
        owner: ctx.sender(),
        spender,
        value: amount
    });

    true
}

entry fun allowance(
    owner: address,
    spender: address,
    allowance: &Allowance,
): u256 {
    if (field::exists_(&allowance.id, owner)) {
        let owner_allowance = field::borrow<ALLOWANCE_, address, Table<address, u256>>(
            &allowance.id,
            owner
        );

        *owner_allowance.borrow(spender)

    } else {
        0
    }
}

entry fun transfer_from(
    sender: address,
    recipient: address,
    amount: u256,
    allowance: &mut Allowance,
    balance: &mut Balance,
    ctx: &TxContext,
): bool {
    if (field::exists_(&allowance.id, sender)) {
        let spender_allowance = field::borrow_mut<ALLOWANCE_, address, Table<address, u256>>(
            &mut allowance.id,
            sender,
        );

        let allowance = spender_allowance.borrow_mut(ctx.sender());
        if (*allowance < amount) {
            abort(ENotAllowed)
        };

        *allowance = *allowance - amount;

        let sender_balance = field::borrow_mut<BALANCE_, address, u256>(
            &mut balance.id,
            sender
        );

        if (*sender_balance < amount) {
            abort(EInssuficientFunds)
        };


        *sender_balance = *sender_balance - amount;

        if (field::exists_(&balance.id, recipient)) {
            let recipient_balance = field::borrow_mut(&mut balance.id, recipient);
            *recipient_balance = *recipient_balance + amount;
        } else {
            field::add(&mut balance.id, recipient, amount);
        };

    } else {
        abort(ENotAllowed)
    };

    emit(Transfer {
        from: sender,
        to: recipient,
        value: amount
    });

    true
}
