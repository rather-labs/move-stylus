module hello_world::erc721_receiver;

// TODO: calculate the actual selector
const ON_ERC721_RECEIVED_SELECTOR: vector<u8> = vector<u8>[0, 1, 2, 3];

//  Whenever an {IERC721} `token_id` token is transferred to this contract via {IERC721-safeTransferFrom}
//  by `operator` from `from`, this function is called.
//      
//  - It must return its Solidity selector to confirm the token transfer.
//  - If any other value is returned or the interface is not implemented by the recipient, the transfer will be reverted.
//  - The selector can be obtained in Solidity with `IERC721Receiver.onERC721Received.selector`.
entry fun on_erc721_received(
    operator: address,
    from: address,
    token_id: u256,
    data: vector<u8>,
): vector<u8> {
    ON_ERC721_RECEIVED_SELECTOR
}