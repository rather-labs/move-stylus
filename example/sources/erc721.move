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

// Errors: ERC721Errors
// ERC721InvalidOwner(owner)
// ERC721NonexistentToken(tokenId)
// ERC721IncorrectOwner(sender, tokenId, owner)
// ERC721InvalidSender(sender)
// ERC721InvalidReceiver(receiver)
// ERC721InsufficientApproval(operator, tokenId)
// ERC721InvalidApprover(approver)
// ERC721InvalidOperator(operator)


const EInssuficientFunds: u64 = 1;
const ENotAllowed: u64 = 2;
const EAlreadyMinted: u64 = 3;
const ETokenNotMinted: u64 = 4;
const EZeroAddress: u64 = 5;

public struct CONTRACT_INFO has key {}
public struct TOTAL_SUPPLY has key {}
public struct OWNERS_ has key {}
public struct BALANCE_ has key {}
public struct TOKEN_APPROVALS has key {}
public struct OPERATOR_APPROVALS has key {}

// Structs:
public struct Info has key {
    id: NamedId<CONTRACT_INFO>,
    name: String,
    symbol: String,
    decimals: u8,
}

public struct TotalSupply has key {
    id: NamedId<TOTAL_SUPPLY>,
    total: u256,
}

public struct Owners has key {
    id: NamedId<OWNERS_>,
}

public struct Balance has key {
    id: NamedId<BALANCE_>,
}

// Holds the approved address (operator)permitted to transfer a particular token.
public struct TokenApprovals has key {
    id: NamedId<TOKEN_APPROVALS>,
}

// Indicates whether an operator has approval to manage all tokens owned by a specific address.
public struct OperatorApprovals has key {
    id: NamedId<OPERATOR_APPROVALS>,
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

// Emitted when `token_id` token is transferred from `from` to `to`.
#[ext(event, indexes = 2)]
public struct Transfer has copy, drop {
    from: address,
    to: address,
    token_id: u256
}

// Constructor
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

    transfer::share_object(Owners {
        id: object::new_named_id<OWNERS_>(),
    });

    transfer::share_object(Balance {
        id: object::new_named_id<BALANCE_>(),
    });

    transfer::share_object(TokenApprovals {
        id: object::new_named_id<TOKEN_APPROVALS>(),
    });

    transfer::share_object(OperatorApprovals {
        id: object::new_named_id<OPERATOR_APPROVALS>(),
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
    token_approvals: &mut TokenApprovals,
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

    // Remove the token_id from the token_approvals table 
    // (TODO: is this the same as setting the zero address as approved?)
    if (field::exists_(&token_approvals.id, token_id)) {
        field::remove<TOKEN_APPROVALS, u256, address>(&mut token_approvals.id, token_id);
    };

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
    token_approvals: &mut TokenApprovals,
    ctx: &mut TxContext,
): bool {
    // Check if the sender is the owner of the token (TODO: or an approved operator...)
    if (owner_of(token_id, owners) != ctx.sender()) {
        abort(ENotAllowed)
    };

    // Approve `to` to operate on `token_id`
    if (field::exists_(&token_approvals.id, token_id)) {
        *field::borrow_mut<TOKEN_APPROVALS, u256, address>(&mut token_approvals.id, token_id) = to;
    } else {
        field::add(
            &mut token_approvals.id,
            token_id,
            to
        );
    };

    // Emit the Approval event
    emit(Approval {
        owner: ctx.sender(),
        approved: to,
        token_id
    });

    true
}

// Returns the account approved for `token_id` token.
//
// Requirements:
// - `token_id` must exist.
entry fun getApproved(token_id: u256, token_approvals: &TokenApprovals): address {
    if (field::exists_(&token_approvals.id, token_id)) {
        *field::borrow<TOKEN_APPROVALS, u256, address>(&token_approvals.id, token_id)
    } else {
        // Return the zero address if `token_id` has no approved operators
        @0x0
    }
}

// Approve or remove `operator` as an operator for the caller.
// Operators can call transferFrom() or safeTransferFrom() for any token owned by the caller.
//
// Requirements:
// - The operator cannot be the address zero.
// 
// Emits an ApprovalForAll event.
entry fun setApprovalForAll(
    operator: address,
    approved: bool,
    operator_approvals: &mut OperatorApprovals,
    ctx: &mut TxContext,
) {
    if (approved) {
        // If approved is true, set or add the entry
        if (field::exists_(&operator_approvals.id, ctx.sender())) {
            *field::borrow_mut<OPERATOR_APPROVALS, address, address>(&mut operator_approvals.id, ctx.sender()) = operator;
        } else {
            field::add(&mut operator_approvals.id, ctx.sender(), operator);
        };
    } else {
        // If approved is false, remove the entry entirely if it exists
        if (field::exists_(&operator_approvals.id, ctx.sender())) {
            field::remove<OPERATOR_APPROVALS, address, address>(&mut operator_approvals.id, ctx.sender());
        };
    };
}

// Returns if the operator is allowed to manage all of the assets of owner.
//
// See setApprovalForAll()
entry fun isApprovedForAll(
    owner: address,
    operator: address,
    operator_approvals: &OperatorApprovals
): bool {
    if (field::exists_(&operator_approvals.id, owner)) {
        *field::borrow<OPERATOR_APPROVALS, address, address>(&operator_approvals.id, owner) == operator
    } else {
        false
    }
}

// Returns whether spender is allowed to manage owner's tokens, or tokenId in particular (ignoring whether it is owned by owner).
//
// Warning: this function assumes that `owner` is the actual owner of `token_id` and does not verify this assumption.
fun isAuthorized_(
    owner: address,
    spender: address,
    token_id: u256,
    token_approvals: &TokenApprovals,
    operator_approvals: &OperatorApprovals
): bool {
    // Check if spender is approved for all tokens of owner
    if (field::exists_(&operator_approvals.id, owner)) {
        if (*field::borrow<OPERATOR_APPROVALS, address, address>(&operator_approvals.id, owner) == spender) {
            return true
        }
    };

    // Check if spender is approved for this specific token
    if (field::exists_(&token_approvals.id, token_id)) {
        return *field::borrow<TOKEN_APPROVALS, u256, address>(&token_approvals.id, token_id) == spender
    };

    false
}

// Checks if `spender` can operate on `token_id`, assuming the provided `owner` is the actual owner.
//
// Reverts if:
// - `spender` does not have approval from `owner` for `token_id`.
// - `spender` does not have approval to manage all of `owner`'s assets.
//
// Warning: this function assumes that `owner` is the actual owner of `token_id` and does not verify this assumption.
fun checkAuthorized_(
    owner: address,
    spender: address,
    token_id: u256,
    token_approvals: &TokenApprovals,
    operator_approvals: &OperatorApprovals
) {
    if (!isAuthorized_(owner, spender, token_id, token_approvals, operator_approvals)) {
        abort(ENotAllowed)
    }
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
    token_approvals: &mut TokenApprovals,
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

    // Increment `to`'s balance by 1
    if (field::exists_(&balance.id, to)) {
        let balance_amount = field::borrow_mut(&mut balance.id, to);
        *balance_amount = *balance_amount + 1;
    } else {
        field::add(&mut balance.id, to, 1);
    };

    // The approval is cleared when the token is transferred.
    if (field::exists_(&token_approvals.id, token_id)) {
        field::remove<TOKEN_APPROVALS, u256, address>(&mut token_approvals.id, token_id);
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
    token_approvals: &mut TokenApprovals,
    operator_approvals: &OperatorApprovals,
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

    // Check if the caller is authorized
    checkAuthorized_(from, ctx.sender(), token_id, token_approvals, operator_approvals);

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

    // Increment `to`'s balance by 1
    if (field::exists_(&balance.id, to)) {
        let balance_amount = field::borrow_mut(&mut balance.id, to);
        *balance_amount = *balance_amount + 1;
    } else {
        field::add(&mut balance.id, to, 1);
    };

    // The approval is cleared when the token is transferred.
    if (field::exists_(&token_approvals.id, token_id)) {
        field::remove<TOKEN_APPROVALS, u256, address>(&mut token_approvals.id, token_id);
    };

    // Emit the Transfer event
    emit(Transfer {
        from,
        to,
        token_id
    });

    true
}
