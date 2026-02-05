// Copyright (c) Mysten Labs, Inc.
// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: Apache-2.0
// Portions of this file were modified by Rather Labs, Inc on 2025-2026.

/// Executing a swap of two objects via a third party, using object wrapping to
/// hand ownership of the objects to swap to the third party without giving them
/// the ability to modify those objects.
module test::trusted_swap;

use stylus::{
    tx_context::TxContext, 
    object::{Self, UID}, 
    transfer::{Self}
};

public struct Object has key, store {
    id: UID,
    scarcity: u8,
    style: u8,
}

public struct SwapRequest has key {
    id: UID,
    owner: address,
    object: Object,
    fee: u64,
}

// === Errors ===

/// Fee is too low for the service
#[error]
const EFeeTooLow: vector<u8> = b"Fee is too low for the service";

/// The two swap requests are not compatible
#[error]
const EBadSwap: vector<u8> = b"The two swap requests are not compatible";

// === Constants ===

const MIN_FEE: u64 = 1000;

// === Public Functions ===

entry fun create_object(scarcity: u8, style: u8, ctx: &mut TxContext) {
    let obj = Object { id: object::new(ctx), scarcity, style };
    transfer::transfer(obj, ctx.sender());
}

entry fun read_object(obj: &Object): &Object {
    obj
}

/// Anyone who owns an `Object` can make it available for swapping, which
/// sends a `SwapRequest` to a `service` responsible for matching swaps.
entry fun request_swap(
    obj: Object,
    service: address,
    fee: u64,
    ctx: &mut TxContext,
) {
    assert!(fee >= MIN_FEE, EFeeTooLow);

    let request = SwapRequest {
        id: object::new(ctx),
        owner: ctx.sender(),
        object: obj,
        fee,
    };

    transfer::transfer(request, service)
}

/// When the service has two swap requests, it can execute them, sending the
/// objects to the respective owners and taking its fee.
entry fun execute_swap(s1: SwapRequest, s2: SwapRequest): u64 {
    let SwapRequest {id: id1, owner: owner1, object: o1, fee: fee1} = s1;
    let SwapRequest {id: id2, owner: owner2, object: o2, fee: fee2} = s2;

    assert!(o1.scarcity == o2.scarcity, EBadSwap);
    assert!(o1.style != o2.style, EBadSwap);

    // Perform the swap
    transfer::transfer(o1, owner2);
    transfer::transfer(o2, owner1);

    // Delete the wrappers
    id1.delete();
    id2.delete();

    // Take the fee and return it
    fee1 + fee2
}