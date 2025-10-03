module hello_world::erc20;

use stylus::event::emit;
use std::ascii::String;
use std::ascii as ascii;
use stylus::tx_context::TxContext;
use stylus::transfer as transfer;
use stylus::object as object;
use stylus::object::NamedId;
use stylus::object::UID;
use stylus::dynamic_field_named_id as field;
use stylus::table::Table;
use stylus::table as table;

const EInssuficientFunds: u64 = 1;

public struct TOTAL_SUPPLY has key {}
public struct CONTRACT_INFO has key {}
public struct ALLOWANCE_ has key {}
public struct BALANCE_ has key {}

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

public struct Transfer has copy, drop {
    from: address,
    to: address,
    value: u256
}

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

public struct AccountAllowance has key {
    id: UID,
}

public fun create(ctx: &mut TxContext) {
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

public fun mint(
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

public fun total_supply(t_supply: &TotalSupply): u256 {
    t_supply.total
}

public fun decimals(contract_info: &Info): u8 {
    contract_info.decimals
}

public fun name(contract_info: &Info): String {
    contract_info.name
}

public fun symbol(contract_info: &Info): String {
    contract_info.symbol
}


public fun balance_of(account: address, balance: &Balance): u256 {
    if (field::exists_(&balance.id, account)) {
        *field::borrow<BALANCE_, address, u256>(&balance.id, account)
    } else {
        0
    }
}

public fun transferr(
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
        abort(EInssuficientFunds);
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

public fun approve(
    spender: address,
    amount: u256,
    allowance: &mut Allowance,
    ctx: &mut TxContext,
): bool {
    let spender_allowance = if (field::exists_(&allowance.id, spender)) {
        field::borrow_mut<ALLOWANCE_, address, Table<address, u256>>(&mut allowance.id, ctx.sender())
    } else {
        field::add(
            &mut allowance.id,
            ctx.sender(),
            table::new<address, u256>(ctx)
        );
        field::borrow_mut<ALLOWANCE_, address, Table<address, u256>>(&mut allowance.id, ctx.sender())
    };

    spender_allowance.add(spender, amount);

    emit(Approval {
        owner: ctx.sender(),
        spender,
        value: amount
    });

    true
}
