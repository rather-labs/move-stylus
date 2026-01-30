// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: BUSL-1.1

module hello_world::erc721_receiver;

// Function selector for the on_erc721_received function according to Remix
// "onERC721Received(address,address,uint256,bytes)": "150b7a02"
const ON_ERC721_RECEIVED_SELECTOR: vector<u8> = x"150b7a02";

//  Whenever an {IERC721} `token_id` token is transferred to this contract via {IERC721-safeTransferFrom}
//  by `operator` from `from`, this function is called.
//
//  - It must return its Solidity selector to confirm the token transfer.
//  - If any other value is returned or the interface is not implemented by the recipient, the transfer will be reverted.
//  - The selector can be obtained in Solidity with `IERC721Receiver.onERC721Received.selector`.
entry fun on_erc721_received(
    _operator: address,
    _from: address,
    _token_id: u256,
    _data: vector<u8>,
): vector<u8> {
    ON_ERC721_RECEIVED_SELECTOR
}
