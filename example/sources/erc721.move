module hello_world::erc721;

use stylus::event::emit;
use stylus::tx_context::TxContext;
use stylus::transfer as transfer;
use stylus::object as object;
use stylus::object::NamedId;
use stylus::dynamic_field_named_id as field;
use erc721Utils::utils as erc721_utils;
use std::ascii as ascii;
use std::ascii::String;
use stylus::sol_types::Bytes4;

public struct COLLECTION_INFO has key {}
public struct TOTAL_SUPPLY has key {}
public struct OWNERS_ has key {}
public struct BALANCES_ has key {}
public struct TOKEN_APPROVALS has key {}
public struct OPERATOR_APPROVALS has key {}

const IERC721_INTERFACE_ID: vector<u8> = vector<u8>[0x80, 0xac, 0x58, 0xcd];
const IERC721_METADATA_INTERFACE_ID: vector<u8> = vector<u8>[0x01, 0xff, 0xc9, 0xa7];

// Structs:
public struct Info has key {
    id: NamedId<COLLECTION_INFO>,
    name: String,
    symbol: String,
    base_uri: String,
}

public struct TotalSupply has key {
    id: NamedId<TOTAL_SUPPLY>,
    total: u256,
}

public struct Owners has key {
    id: NamedId<OWNERS_>,
}

public struct Balances has key {
    id: NamedId<BALANCES_>,
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
#[ext(event, indexes = 3)]
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
#[ext(event, indexes = 3)]
public struct Transfer has copy, drop {
    from: address,
    to: address,
    token_id: u256
}

// Errors:
const EInvalidOwner: u64 = 1;
const ENonExistentToken: u64 = 2;
const EIncorrectOwner: u64 = 3;
const EInvalidReceiver: u64 = 4;
const EInvalidApprover: u64 = 5;
const EInvalidOperator: u64 = 6;

// Methods:

// Constructor
entry fun init(_ctx: &mut TxContext) {
    transfer::freeze_object(Info {
        id: object::new_named_id<COLLECTION_INFO>(),
        name: ascii::string(b"Moving Stylus"),
        symbol: ascii::string(b"MST"),
        base_uri: ascii::string(b"https://examplerc721.com/token/"),
    });

    transfer::share_object(TotalSupply {
        id: object::new_named_id<TOTAL_SUPPLY>(),
        total: 0,
    });

    transfer::share_object(Owners {
        id: object::new_named_id<OWNERS_>(),
    });

    transfer::share_object(Balances {
        id: object::new_named_id<BALANCES_>(),
    });

    transfer::share_object(TokenApprovals {
        id: object::new_named_id<TOKEN_APPROVALS>(),
    });

    transfer::share_object(OperatorApprovals {
        id: object::new_named_id<OPERATOR_APPROVALS>(),
    });
}

entry fun supports_interface(interface_id: Bytes4): bool {
    (interface_id.as_vec() == IERC721_INTERFACE_ID) || (interface_id.as_vec() == IERC721_METADATA_INTERFACE_ID)
}

// Returns the number of tokens in `owner`'s account.
entry fun balance_of(owner: address, balances: &Balances): u256 {
    if (owner == @0x0) {
        abort(EInvalidOwner)
    };

    if (field::exists_(&balances.id, owner)) {
        *field::borrow<BALANCES_, address, u256>(&balances.id, owner)
    } else {
        0
    }
}

// Returns the owner of the `token_id` token.
//
// Requirements:
// - `token_id` must exist.
entry fun owner_of(token_id: u256, owners: &Owners): address {
    require_owned_(token_id, owners)
}

entry fun name(contract_info: &Info): String {
    contract_info.name
}

entry fun symbol(contract_info: &Info): String {
    contract_info.symbol
}

entry fun token_URI(token_id: u256, contract_info: &Info): String {
    let mut token_url = std::ascii::string(*contract_info.base_uri.as_bytes());
    token_url.append(std::ascii::string(*token_id.to_string().as_bytes()));
    token_url
}

entry fun total_supply(t_supply: &TotalSupply): u256 {
    t_supply.total
}

// Grants permission to `to` transfer `token_id` to another account.
// The approval is automatically cleared when the token is transferred.
//
// Only a single account can be approved at a time, so approving the zero address clears previous approvals.
//
// Requirements:
// - The caller must own the token or be an approved operator.
// - The `token_id` must exist.
//
// Emits an Approval event.
entry fun approve(
    to: address,
    token_id: u256,
    owners: &Owners,
    token_approvals: &mut TokenApprovals,
    operator_approvals: &OperatorApprovals,
    ctx: &TxContext,
) {
    // Get the owner of the token. Aborts if not minted.
    let owner = require_owned_(token_id, owners);

    // Check if tx sender is the owner of the token or has approval to manage all of the owner's tokens.
    if (owner != ctx.sender() && !is_approved_for_all(owner, ctx.sender(), operator_approvals)) {
        abort(EInvalidApprover)
    };

    if (to != @0x0) {
        // Set the approval for `token_id` to `to`
        if (field::exists_(&token_approvals.id, token_id)) {
            *field::borrow_mut<TOKEN_APPROVALS, u256, address>(&mut token_approvals.id, token_id) = to;
        } else {
            field::add(
                &mut token_approvals.id,
                token_id,
                to
            );

        };
    } else {
        // Remove the approval for `token_id` when `to` is the zero address
        field::remove_if_exists<TOKEN_APPROVALS, u256, address>(&mut token_approvals.id, token_id);
    };

    emit(Approval {
        owner,
        approved: to,
        token_id
    });
}

// Returns the account approved for `token_id` token.
//
// Requirements:
// - `token_id` must exist.
entry fun get_approved(token_id: u256, owners: &Owners, token_approvals: &TokenApprovals): address {
    require_owned_(token_id, owners);

    if (field::exists_(&token_approvals.id, token_id)) {
        *field::borrow<TOKEN_APPROVALS, u256, address>(&token_approvals.id, token_id)
    } else {
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
entry fun set_approval_for_all(
    operator: address,
    approved: bool,
    operator_approvals: &mut OperatorApprovals,
    ctx: &TxContext,
) {
    if (operator == @0x0) {
        abort(EInvalidOperator)
    };

    if (approved) {
        // If approved is true, set or add the entry
        if (field::exists_(&operator_approvals.id, ctx.sender())) {
            *field::borrow_mut<OPERATOR_APPROVALS, address, address>(&mut operator_approvals.id, ctx.sender()) = operator;
        } else {
            field::add(&mut operator_approvals.id, ctx.sender(), operator);
        };
    } else {
        // If approved is false, remove the entry entirely if it exists
        field::remove_if_exists<OPERATOR_APPROVALS, address, address>(&mut operator_approvals.id, ctx.sender());
    };
    emit(ApprovalForAll {
        owner: ctx.sender(),
        operator,
        approved
    });
}

// Returns if the operator is allowed to manage all of the assets of owner.
//
// See setApprovalForAll()
entry fun is_approved_for_all(
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

// Mints `token_id` and transfers it to `to`.
//
// Requirements:
// - `to` cannot be the zero address.
// - `token_id` must not exist.
//
// Emits a Transfer event.
entry fun mint(
    to: address,
    token_id: u256,
    owners: &mut Owners,
    balances: &mut Balances,
    total_supply: &mut TotalSupply,
) {
    // Check if the token has already been minted
    if (field::exists_(&owners.id, token_id)) {
        abort(EInvalidOperator)
    } else {
        field::add(&mut owners.id, token_id, to);
    };

    // Increment the balance of `to` by 1
    if (field::exists_(&balances.id, to)) {
        // TODO:  If the type is not specified,
        // calls to balance_of() hit an unreachable (-3200)
        let to_balance = field::borrow_mut<BALANCES_, address, u256>(&mut balances.id, to);
        *to_balance = *to_balance + 1;
    } else {
        field::add<BALANCES_, address, u256>(&mut balances.id, to, 1);
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
// Emits a Transfer event.
entry fun burn(
    token_id: u256,
    owners: &Owners,
    balances: &mut Balances,
    token_approvals: &mut TokenApprovals,
    total_supply: &mut TotalSupply,
) {
    // Check if the token has been minted
    if (!field::exists_(&owners.id, token_id)) {
        abort(ENonExistentToken)
    };

    // Get the owner of the token
    let owner = owner_of(token_id, owners);

    // We are assuming here that the owner's balance entry already exists.
    let owner_balance = field::borrow_mut<BALANCES_, address, u256>(&mut balances.id, owner);

    // Decrement the balance of the owner by 1
    *owner_balance = *owner_balance - 1;

    // Decrement the total supply by 1
    total_supply.total = total_supply.total - 1;

    // Remove the token_id from the token_approvals table
    // (TODO: is this the same as setting the zero address as approved?)
    field::remove_if_exists<TOKEN_APPROVALS, u256, address>(&mut token_approvals.id, token_id);

    emit(Transfer {
        from: owner,
        to: @0x0,
        token_id
    });
}

// Transfers `token_id` from `from` to `to`. It imposes no restrictions on msg.sender.
//
// Requirements:
// - `to` cannot be the zero address.
// - `token_id` token must be owned by `from`.
//
// Emits a Transfer event.
// TODO: should be private
entry fun transfer(
    from: address,
    to: address,
    token_id: u256,
    owners: &mut Owners,
    balances: &mut Balances,
    token_approvals: &mut TokenApprovals,
) {
    // Check if `to` is the zero address
    if (to == @0x0) {
        abort(EInvalidReceiver)
    };

    if (field::exists_(&owners.id, token_id)) {
        // Transfer the ownership of the token to `to`
        let owner = field::borrow_mut<OWNERS_, u256, address>(&mut owners.id, token_id);
        if (owner != from) {
            abort(EIncorrectOwner)
        };
        *owner = to;

        // Decrement from's balance by 1
        let from_balance = field::borrow_mut<BALANCES_, address, u256>(&mut balances.id, from);
        *from_balance = *from_balance - 1;

        // Increment to's balance by 1
        if (field::exists_(&balances.id, to)) {
            let to_balance = field::borrow_mut<BALANCES_, address, u256>(&mut balances.id, to);
            *to_balance = *to_balance + 1;
        } else {
            field::add<BALANCES_, address, u256>(&mut balances.id, to, 1);
        };

        // Clear approval for `token_id`
        field::remove_if_exists<TOKEN_APPROVALS, u256, address>(&mut token_approvals.id, token_id);
    } else {
        abort(ENonExistentToken)
    };

    emit(Transfer {
        from,
        to,
        token_id
    });
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
    balances: &mut Balances,
    token_approvals: &mut TokenApprovals,
    operator_approvals: &OperatorApprovals,
    ctx: &TxContext,
) {
    check_authorized_(from, ctx.sender(), token_id, token_approvals, operator_approvals);
    transfer(from, to, token_id, owners, balances, token_approvals);
}

entry fun safeTransferFrom(
    from: address,
    to: address,
    token_id: u256,
    data: vector<u8>,
    owners: &mut Owners,
    balances: &mut Balances,
    token_approvals: &mut TokenApprovals,
    operator_approvals: &OperatorApprovals,
    ctx: &TxContext,
) {
        transfer_from(from, to, token_id, owners, balances, token_approvals, operator_approvals, ctx);
        erc721_utils::check_on_erc721_received(ctx.sender(), from, to, token_id, data);
    }

// Private methods:

// Returns the owner of the `token_id`. Does NOT revert if token doesn't exist.
//
// IMPORTANT: Any overrides to this function that add ownership of tokens not tracked by the
// core ERC-721 logic MUST be matched with the use of {increase_balance_} to keep balances
// consistent with ownership. The invariant to preserve is that for any address `a` the value returned by
// `balanceOf(a)` must be equal to the number of tokens such that `owner_of_(token_id)` is `a`.
fun owner_of_(
    token_id: u256,
    owners: &Owners,
): address {
    if (field::exists_(&owners.id, token_id)) {
        *field::borrow<OWNERS_, u256, address>(&owners.id, token_id)
    } else {
        @0x0
    }
}

// Reverts if the `token_id` doesn't have a current owner (it hasn't been minted, or it has been burned).
// Returns the owner.
//
// Overrides to ownership logic should be done to {owner_of_}.
fun require_owned_(
    token_id: u256,
    owners: &Owners,
): address {
    let owner = owner_of_(token_id, owners);
    if (owner == @0x0) {
        abort(ENonExistentToken)
    };
    owner
}

// Returns whether spender is allowed to manage owner's tokens, or tokenId in particular (ignoring whether it is owned by owner).
//
// Warning: this function assumes that `owner` is the actual owner of `token_id` and does not verify this assumption.
fun is_authorized_(
    owner: address,
    spender: address,
    token_id: u256,
    token_approvals: &TokenApprovals,
    operator_approvals: &OperatorApprovals
): bool {
    // Check if spender is the owner of the token
    if (owner == spender) {
        return true
    };

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
fun check_authorized_(
    owner: address,
    spender: address,
    token_id: u256,
    token_approvals: &TokenApprovals,
    operator_approvals: &OperatorApprovals
) {
    if (!is_authorized_(owner, spender, token_id, token_approvals, operator_approvals)) {
        abort(EInvalidApprover)
    }
}
