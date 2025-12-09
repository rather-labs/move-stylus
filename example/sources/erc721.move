module hello_world::erc721;

use stylus::event::emit;
use std::ascii::String;
use std::ascii as ascii;
use stylus::tx_context::TxContext;
use stylus::transfer as transfer;
use stylus::object as object;
use stylus::object::NamedId;
use stylus::dynamic_field_named_id as field;
use stylus::table::Table;
use stylus::table as table;

// Methods:
// function balanceOf(address _owner) external view returns (uint256);
// function ownerOf(uint256 _tokenId) external view returns (address);
// function safeTransferFrom(address _from, address _to, uint256 _tokenId, bytes data) external payable;
// function safeTransferFrom(address _from, address _to, uint256 _tokenId) external payable;
// function transferFrom(address _from, address _to, uint256 _tokenId) external payable;
// function approve(address _approved, uint256 _tokenId) external payable;
// function setApprovalForAll(address _operator, bool _approved) external;
// function getApproved(uint256 _tokenId) external view returns (address);
// function isApprovedForAll(address _owner, address _operator) external view returns (bool);

// Events:
// event Transfer(address indexed _from, address indexed _to, uint256 indexed _tokenId);
// event Approval(address indexed _owner, address indexed _approved, uint256 indexed _tokenId);
// event ApprovalForAll(address indexed _owner, address indexed _operator, bool _approved);


const EInssuficientFunds: u64 = 1;
const ENotAllowed: u64 = 2;
const EAlreadyMinted: u64 = 3;
const ETokenNotMinted: u64 = 4;
const EZeroAddress: u64 = 5;

public struct TOTAL_SUPPLY has key {}
public struct CONTRACT_INFO has key {}
public struct ALLOWANCE_ has key {}
public struct BALANCE_ has key {}
public struct OWNERS_ has key {}

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

// Emitted when `token_id` token is transferred from `from` to `to`.
#[ext(event, indexes = 2)]
public struct Transfer has copy, drop {
    from: address,
    to: address,
    token_id: u256
}

// Emitted when `owner` enables `approved` to manage the `token_id` token.
#[ext(event, indexes = 2)]
public struct Approval has copy, drop {
    owner: address,
    approved: address,
    token_id: u256
}

// Emitted when `owner` enables or disables (`approved`) `operator` to manage all of its assets.
#[ext(event, indexes = 2)]
public struct ApprovalForAll has copy, drop {
    owner: address,
    operator: address,
    approved: bool
}

public struct Balance has key {
    id: NamedId<BALANCE_>,
}

public struct Allowance has key {
    id: NamedId<ALLOWANCE_>,
}

public struct Owners has key {
    id: NamedId<OWNERS_>,
}

fun init(_ctx: &mut TxContext) {
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
    token_id: u256,
    owners: &mut Owners,
    balance: &mut Balance,
    total_supply: &mut TotalSupply,
) {
    // Check if the token has already been minted
    if (field::exists_(&owners.id, token_id)) {
        abort(EAlreadyMinted)
    } else {
        field::add(&mut owners.id, token_id, to);
    };

    // Increment the balance of `to` by 1
    if (field::exists_(&balance.id, to)) {
        let balance_amount = field::borrow_mut(&mut balance.id, to);
        *balance_amount = *balance_amount + 1;
    } else {
        field::add(&mut balance.id, to, 1);
    };

    // Increment the total supply by 1
    total_supply.total = total_supply.total + 1;

    emit(Transfer {
        from: @0x0,
        to,
        token_id
    });
}

// Destroys `token_id`. The approval is cleared when the token is burned. 
// This is an internal function that does not check if the sender is authorized to operate on the token.
//
// Requirements:
// - `token_id` must exist.
// 
// Emits a IERC6909.Transfer event.
entry fun burn(
    token_id: u256,
    owners: &mut Owners,
    balance: &mut Balance,
    allowance: &mut Allowance,
    total_supply: &mut TotalSupply,
) {
    // Check if the token has been minted
    if (!field::exists_(&owners.id, token_id)) {
        abort(ETokenNotMinted)
    };

    // Get the owner of the token
    let owner = owner_of(token_id, owners);

    // We are assuming here that the owner's balance entry already exists.
    let balance_amount = field::borrow_mut(&mut balance.id, owner);

    // Decrement the balance of the owner by 1
    *balance_amount = *balance_amount - 1;

    // Decrement the total supply by 1
    total_supply.total = total_supply.total - 1;

    // Get the owner's allowance table
    let owner_allowance = field::borrow_mut<ALLOWANCE_, address, Table<u256, vector<address>>>(
        &mut allowance.id,
        owner
    );

    // Remove the token_id from the owner's allowance table
    owner_allowance.remove(token_id);

    emit(Transfer {
        from: owner,
        to: @0x0,
        token_id
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

// Returns the number of tokens in `owner`'s account.
entry fun balance_of(owner: address, balance: &Balance): u256 {
    if (field::exists_(&balance.id, owner)) {
        *field::borrow<BALANCE_, address, u256>(&balance.id, owner)
    } else {
        0
    }
}

// Returns the owner of the `token_id` token.
//
// Requirements:
// - `token_id` must exist.
entry fun owner_of(token_id: u256, owners: &Owners): address {
    if (field::exists_(&owners.id, token_id)) {
        *field::borrow<OWNERS_, u256, address>(&owners.id, token_id)
    } else {
        abort(ETokenNotMinted)
    }
}

// Grants permission to `to` transfer `token_id` to another account. 
// The approval is automatically cleared when the token is transferred.
//
// Only a single account can be approved at a time, so approving the zero address clears previous approvals (?)
//
// Requirements:
// - The caller must own the token or be an approved operator.
// - The tokenId must exist.
// 
// Emits an ERC20.Approval event.
entry fun approve(
    to: address,
    token_id: u256,
    owners: &mut Owners,
    allowance: &mut Allowance,
    ctx: &mut TxContext,
): bool {
    // Check if the sender is the owner of the token (TODO: or an approved operator...)
    if (owner_of(token_id, owners) != ctx.sender()) {
        abort(ENotAllowed)
    };

    // Get the sender's allowance table
    let sender_allowance = if (field::exists_(&allowance.id, ctx.sender())) {
        field::borrow_mut<ALLOWANCE_, address, Table<u256, vector<address>>>(&mut allowance.id, ctx.sender())
    } else {
        field::add(
            &mut allowance.id,
            ctx.sender(),
            table::new<address, u256>(ctx)
        );
        field::borrow_mut<ALLOWANCE_, address, Table<u256, vector<address>>>(&mut allowance.id, ctx.sender())
    };

    // Get the allowance for the `token_id` and push the `to` address to it
    if (sender_allowance.contains(token_id)) {
        let allowance = sender_allowance.borrow_mut(token_id);
        vector::push_back(allowance, to);
    } else {
        sender_allowance.add(token_id, vector[to]);
    };

    emit(Approval {
        owner: ctx.sender(),
        approved: to,
        token_id
    });

    true
}

// Transfers `token_id` from `from` to `to`. It imposes no restrictions on msg.sender.
//
// Requirements:
// - `to` cannot be the zero address.
// - `token_id` token must be owned by `from`.
//
// Emits a Transfer event.
entry fun transfer(
    from: address,
    to: address,
    token_id: u256,
    owners: &mut Owners,
    balance: &mut Balance,
    allowance: &mut Allowance,
): bool {
    // Check if `to` is the zero address
    if (to == @0x0) {
        abort(EZeroAddress)
    };

    // Check if `from` is the owner of the token
    let owner = owner_of(token_id, owners);
    if (owner != from) {
        abort(ENotAllowed)
    };

    // Remove `token_id` from `from`'s owners table
    field::remove<OWNERS_, u256, address>(&mut owners.id, token_id);

    // Get the owner's balance
    let owner_balance = field::borrow_mut<BALANCE_, address, u256>(
        &mut balance.id,
        owner
    );

    // Decrement the owner's balance by 1
    *owner_balance = *owner_balance - 1;

    // Add `token_id` to `to`'s owners table
    field::add(&mut owners.id, token_id, to);

    // Add `token_id` to `to`'s balance table
    if (field::exists_(&balance.id, to)) {
        let balance_amount = field::borrow_mut(&mut balance.id, to);
        *balance_amount = *balance_amount + 1;
    } else {
        field::add(&mut balance.id, to, 1);
    };

    // The approval is cleared when the token is transferred.
    if (field::exists_(&allowance.id, owner)) {
        let owner_allowance = field::borrow_mut<ALLOWANCE_, address, Table<u256, vector<address>>>(
            &mut allowance.id,
            owner
        );
        owner_allowance.remove(token_id);
    };

    // Emit the Transfer event
    emit(Transfer {
        from,
        to,
        token_id
    });

    true
}

// Transfers `token_id` token from `from` to `to`.
//
// Requirements:
// - `from` cannot be the zero address.
// - `to` cannot be the zero address.
// - `token_id` token must be owned by `from`.
// - If the caller is not `from`, it must be approved to move this token by either approve() or setApprovalForAll().
// 
// Emits a Transfer event.
//
// Note that the caller is responsible to confirm that the recipient is capable of receiving ERC-721 or else they may be permanently lost.
// Usage of safeTransferFrom() prevents loss, though the caller must understand this adds an external call which potentially creates a reentrancy vulnerability.
entry fun transfer_from(
    from: address,
    to: address,
    token_id: u256,
    owners: &mut Owners,
    balance: &mut Balance,
    allowance: &mut Allowance,
    ctx: &TxContext,
): bool {
    // Check if `from` is the zero address
    if (from == @0x0) {
        abort(EZeroAddress)
    };

    // Check if `to` is the zero address
    if (to == @0x0) {
        abort(EZeroAddress)
    };

    // Check if `from` is the owner of the token
    if (owner_of(token_id, owners) != from) {
        abort(ENotAllowed)
    };

    // Check if the caller is the owner of the token, or if it is approved to transfer the token
    if (from != ctx.sender()) {
        if (!field::exists_(&allowance.id, ctx.sender())) {
            abort(ENotAllowed)
        };
        let owner_allowance = field::borrow_mut<ALLOWANCE_, address, Table<u256, vector<address>>>(
            &mut allowance.id,
            from
        );
        if (!owner_allowance.contains(token_id)) {
            abort(ENotAllowed)
        };
        let token_allowance = owner_allowance.borrow_mut(token_id);
        if (!vector::contains(token_allowance, &ctx.sender())) {
            abort(ENotAllowed)
        };
    };

    // Remove `token_id` from `from`'s owners table
    field::remove<OWNERS_, u256, address>(&mut owners.id, token_id);

    // Get the owner's balance
    let owner_balance = field::borrow_mut<BALANCE_, address, u256>(
        &mut balance.id,
        from
    );

    // Decrement the owner's balance by 1
    *owner_balance = *owner_balance - 1;

    // Add `token_id` to `to`'s owners table
    field::add(&mut owners.id, token_id, to);

    // Add `token_id` to `to`'s balance table
    if (field::exists_(&balance.id, to)) {
        let balance_amount = field::borrow_mut(&mut balance.id, to);
        *balance_amount = *balance_amount + 1;
    } else {
        field::add(&mut balance.id, to, 1);
    };

    // The approval is cleared when the token is transferred.
    if (field::exists_(&allowance.id, from)) {
        let owner_allowance = field::borrow_mut<ALLOWANCE_, address, Table<u256, vector<address>>>(
                &mut allowance.id,
                from
            );
        owner_allowance.remove(token_id);
    };

    // Emit the Transfer event
    emit(Transfer {
        from,
        to,
        token_id
    });

    true
}
