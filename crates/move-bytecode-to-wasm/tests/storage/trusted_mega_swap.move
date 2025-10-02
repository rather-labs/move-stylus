// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// Executing a swap of two objects via a third party, using object wrapping to
/// hand ownership of the objects to swap to the third party without giving them
/// the ability to modify those objects.
module test::trusted_mega_swap;

use stylus::tx_context::TxContext;
use stylus::object as object;
use stylus::object::UID;
use stylus::transfer as transfer;

public struct ObjectWrapper has key, store {
    id: UID,
    object: Object,
    vec_object: vector<Object>,
}

public struct Object has key, store {
    id: UID,
    scarcity: u8,
    style: u8,
}

public struct SwapRequest has key {
    id: UID,
    owner: address,
    wrapper: ObjectWrapper,
    fee: u64,
}

// === Errors ===

/// Fee is too low for the service
const EFeeTooLow: u64 = 2;

/// The two swap requests are not compatible
const EBadSwap: u64 = 3;

// === Constants ===

const MIN_FEE: u64 = 1000;

// === Public Functions ===

public fun create_object(scarcity: u8, style: u8, ctx: &mut TxContext) {
    let obj = Object { id: object::new(ctx), scarcity, style };
    transfer::transfer(obj, ctx.sender());
}

public fun read_object(obj: &Object): &Object {
    obj
}

/// Anyone who owns an `Object` can make it available for swapping, which
/// sends a `SwapRequest` to a `service` responsible for matching swaps.
public fun request_mega_swap(
    obj1: Object,
    obj2: Object,
    obj3: Object,
    service: address,
    fee: u64,
    ctx: &mut TxContext,
) {
    assert!(fee >= MIN_FEE, EFeeTooLow);

    let wrapper = ObjectWrapper { id: object::new(ctx), object: obj1, vec_object: vector[obj2, obj3] };

    let request = SwapRequest {
        id: object::new(ctx),
        owner: ctx.sender(),
        wrapper,
        fee,
    };

    transfer::transfer(request, service)
}

/// When the service has two swap requests, it can execute them, sending the
/// objects to the respective owners and taking its fee.
public fun execute_mega_swap(s1: SwapRequest, s2: SwapRequest): u64 {
    let SwapRequest {id: id1_swap, owner: owner1, wrapper: w1, fee: fee1} = s1;
    let SwapRequest {id: id2_swap, owner: owner2, wrapper: w2, fee: fee2} = s2;

    let ObjectWrapper {id: id1_wrapper, object: o1, vec_object: mut v1} = w1;
    let ObjectWrapper {id: id2_wrapper, object: o2, vec_object: mut v2} = w2;

    assert!(o1.scarcity == o2.scarcity, EBadSwap);
    assert!(o1.style != o2.style, EBadSwap);

    // Perform the swap
    transfer::transfer(o1, owner2);
    transfer::transfer(o2, owner1);

    // Transfer all objects from v1 to owner2
    while (vector::length(&v1) > 0) {
        let obj = vector::pop_back(&mut v1);
        transfer::transfer(obj, owner2);
    };
    
    // Transfer all objects from v2 to owner1
    while (vector::length(&v2) > 0) {
        let obj = vector::pop_back(&mut v2);
        transfer::transfer(obj, owner1);
    };

    // Destroy the empty vectors
    vector::destroy_empty(v1);
    vector::destroy_empty(v2);

    // Delete the swap requests
    id1_swap.delete();
    id2_swap.delete();

    // Delete the wrappers
    id1_wrapper.delete();
    id2_wrapper.delete();

    // Take the fee and return it
    fee1 + fee2
}