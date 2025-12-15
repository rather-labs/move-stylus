module erc721ReceiverCall::erc721ReceiverCall;

use stylus::contract_calls::{ContractCallResult, CrossContractCall};

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

#[ext(external_call, view)]
public native fun on_erc721_received(
    self: &ERC721Receiver,
    operator: address,
    from: address,
    token_id: u256,
    data: vector<u8>,
): ContractCallResult<vector<u8>>;