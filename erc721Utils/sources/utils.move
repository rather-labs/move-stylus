// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

module erc721Utils::utils;

use stylus::{
    account as account, 
    contract_calls as ccc, 
    contract_calls::{CrossContractCall, ContractCallResult}
};

const EInvalidReceiver: u64 = 1;

// Function selector for the on_erc721_received function according to Remix
// "onERC721Received(address,address,uint256,bytes)": "150b7a02"
const ON_ERC721_RECEIVED_SELECTOR: vector<u8> = vector<u8>[0x15, 0x0b, 0x7a, 0x02];

// Public accessor for the selector, since constants are module-internal in Move
public fun on_erc721_received_selector(): vector<u8> {
    ON_ERC721_RECEIVED_SELECTOR
}

#[ext(external_call)]
public struct ERC721Receiver(CrossContractCall) has drop;

public fun new(configuration: CrossContractCall): ERC721Receiver {
    ERC721Receiver(configuration)
}

#[ext(external_call(view))]
public native fun on_erc721_received(
    self: &ERC721Receiver,
    operator: address,
    from: address,
    token_id: u256,
    data: vector<u8>,
): ContractCallResult<vector<u8>>;

// Performs an acceptance check for the provided `operator` by calling {IERC721Receiver-onERC721Received}
// on the `to` address. The `operator` is generally the address that initiated the token transfer (i.e. `msg.sender`).
// The acceptance call is not executed and treated as a no-op if the target address doesn't contain code (i.e. an EOA).
// Otherwise, the recipient must implement {IERC721Receiver-onERC721Received} and return the acceptance magic value to accept
// the transfer.
public fun check_on_erc721_received(
    operator: address,
    from: address,
    to: address,
    token_id: u256,
    data: vector<u8>,
) {
    if (account::get_account_code_size(to) != 0) {
        let erc721_receiver = new(ccc::new(to));

        let erc721_receiver_call = erc721_receiver.on_erc721_received(operator, from, token_id, data);
        if (!erc721_receiver_call.succeded()) {
            abort(EInvalidReceiver)
        };
        
        if (erc721_receiver_call.get_result() != on_erc721_received_selector()) 
            abort(EInvalidReceiver)
        };
    }
}