use alloy::primitives::address;
use alloy::primitives::{FixedBytes, U256};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolEvent;
use alloy::{primitives::Address, providers::ProviderBuilder, sol, transports::http::reqwest::Url};
use dotenv::dotenv;
use eyre::eyre;
use std::str::FromStr;
use std::sync::Arc;

sol!(
    #[sol(rpc)]
    #[allow(missing_docs)]
    contract Erc721 {
        #[derive(Debug)]
        event Approval(address indexed owner, address indexed approved, uint256 tokenId);

        #[derive(Debug)]
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

        #[derive(Debug)]
        event Transfer(address indexed from, address indexed to, uint256 tokenId);

        function constructor() public view;
        function mint(address to, uint256 tokenId) external view;
        function burn(uint256 tokenId) external view;
        function balanceOf(address owner) public view returns (uint256);
        function ownerOf(uint256 tokenId) public view returns (address);
        function totalSupply() external view returns (uint256);
        function transfer(address from, address to, uint256 tokenId) external;
        function approve(address to, uint256 tokenId) external;
        function getApproved(uint256 tokenId) external view returns (address);
        function setApprovalForAll(address operator, bool approved) external;
        function isApprovedForAll(address owner, address operator) external view returns (bool);
        function transferFrom(address from, address to, uint256 tokenId) external;
        function safeTransferFrom(address from, address to, uint256 tokenId, uint8[] data);
        function name() external view returns (string);
        function symbol() external view returns (string);
        function tokenUri(uint256 tokenId) external view returns (string);
        function supportsInterface(bytes4 interfaceId) external view returns (bool);
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let priv_key_2 =
        std::env::var("PRIV_KEY_2").map_err(|_| eyre!("No {} env var set", "PRIV_KEY_2"))?;

    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_ERC721")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_ERC721"))?;

    let receiver_contract_address = std::env::var("CONTRACT_ADDRESS_ERC721_RECEIVER")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_ERC721_RECEIVER"))?;
    let receiver_address = Address::from_str(&receiver_contract_address)?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;
    let sender = signer.address();

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = Erc721::new(address, provider.clone());

    // Second sender
    let signer_2 = PrivateKeySigner::from_str(&priv_key_2)?;
    let sender_2 = signer_2.address();

    let provider_2 = Arc::new(
        ProviderBuilder::new()
            .wallet(signer_2)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let example_2 = Erc721::new(address, provider_2.clone());

    let address_2 = address!("0xcafecafecafecafecafecafecafecafecafecafe");

    println!("====================");
    println!("Creating a new erc721");
    println!("====================");
    let _pending_tx_ = example.constructor().send().await?;
    println!("Created!");

    println!("\n====================");
    println!("  Contract Info");
    println!("====================");

    let res = example.totalSupply().call().await?;
    println!("Total Supply = {res}");

    let res = example.name().call().await?;
    println!("name = {res}");

    let res = example.symbol().call().await?;
    println!("symbol = {res}");

    println!("\n====================");
    println!("  Mint");
    println!("====================");

    let res = example.balanceOf(sender).call().await?;
    println!("Balance of sender = {res}");

    let res = example.balanceOf(sender_2).call().await?;
    println!("Balance of sender_2 = {res}");

    let token_id_1 = U256::from(1);
    let token_id_2 = U256::from(2);
    let token_id_3 = U256::from(3);

    println!("Minting token {token_id_1} to target address");
    let pending_tx = example.mint(sender, token_id_1).send().await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("Mint events");
    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Erc721::Transfer::decode_log(&primitive_log)?;
        println!("Mint event log {} {:#?}", index + 1, decoded_event);
    }

    println!("Minting token {token_id_2} to target address");
    let pending_tx = example.mint(sender, token_id_2).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Erc721::Transfer::decode_log(&primitive_log)?;
        println!("Mint event log {} {:#?}", index + 1, decoded_event);
    }

    println!("Minting token {token_id_3} to address_1");
    let pending_tx = example.mint(sender_2, token_id_3).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Erc721::Transfer::decode_log(&primitive_log)?;
        println!("Mint event log {} {:#?}", index + 1, decoded_event);
    }

    let res = example.totalSupply().call().await?;
    println!("Total Supply after mint = {res}");

    let res = example.balanceOf(sender).call().await?;
    println!("Balance of sender = {res}");

    let res = example.balanceOf(sender_2).call().await?;
    println!("Balance of sender_2 = {res}");

    let res = example.ownerOf(token_id_1).call().await?;
    println!("Owner of token {token_id_1} = {res}");

    let res = example.ownerOf(token_id_2).call().await?;
    println!("Owner of token {token_id_2} = {res}");

    let res = example.ownerOf(token_id_3).call().await?;
    println!("Owner of token {token_id_3} = {res}");

    println!("\n====================");
    println!("  Burn");
    println!("====================");

    println!("Burning token {token_id_2} from {sender}");

    let pending_tx = example.burn(token_id_2).send().await?;
    let receipt = pending_tx.get_receipt().await?;

    println!("Burn events");
    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Erc721::Transfer::decode_log(&primitive_log)?;
        println!("Burn event log {} {:#?}", index + 1, decoded_event);
    }

    let res = example.totalSupply().call().await?;
    println!("Total Supply after burn = {res}");

    let res = example.balanceOf(sender).call().await?;
    println!("Balance of sender = {res}");

    println!("\n====================");
    println!("  Transfer");
    println!("====================");

    println!("Transfering token {token_id_1} from {sender} to {sender_2}");

    let pending_tx = example
        .transfer(sender, sender_2, token_id_1)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Erc721::Transfer::decode_log(&primitive_log)?;
        println!("Transfer event log {} {:#?}", index + 1, decoded_event);
    }

    let res = example.balanceOf(sender).call().await?;
    println!("  Balance of origin address {sender} after transaction = {res}");
    let res = example.balanceOf(sender_2).call().await?;
    println!("  Balance of target address {sender_2} after transaction = {res}");
    let res = example.ownerOf(token_id_1).call().await?;
    println!("  Owner of token {token_id_1} after transaction = {res}");

    println!("\n==============================");
    println!("  Approval and transfer from");
    println!("================================");

    println!("Allow {sender} to transfer token {token_id_3} from {sender_2}");
    let res = example.getApproved(token_id_3).call().await?;
    println!("  Current approved address for token {token_id_3} = {res}");

    println!();

    println!("Executing approve...");
    let pending_tx = example_2.approve(sender, token_id_3).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Approval events");

    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        if let Ok(decoded_event) = Erc721::Approval::decode_log(&primitive_log) {
            println!("Approval event log {} {:#?}", index + 1, decoded_event);
        }
    }

    println!();

    println!("Checking approvals and balances");
    let res = example.getApproved(token_id_3).call().await?;
    println!("  Current approved address for token {token_id_3} = {res}");
    let res = example.balanceOf(sender).call().await?;
    println!("  Current balance of {sender} = {res}");
    let res = example.balanceOf(sender_2).call().await?;
    println!("  Current balance of {sender_2} = {res}");

    println!();

    println!("Using transferFrom:");
    println!(" from: {sender_2}");
    println!(" to: {address_2}");
    println!(" token_id: {token_id_3}");
    let pending_tx = example
        .transferFrom(sender_2, address_2, token_id_3)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Transfer events");
    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Erc721::Transfer::decode_log(&primitive_log)?;
        println!("Transfer event log {} {:#?}", index + 1, decoded_event);
    }

    println!();

    println!("Checking approvals and balances after transfer");
    let res = example.getApproved(token_id_3).call().await?;
    println!("  Current approved address for token {token_id_3} = {res}");
    let res = example.balanceOf(sender).call().await?;
    println!("  Current balance of {sender} = {res}");
    let res = example.balanceOf(sender_2).call().await?;
    println!("  Current balance of {sender_2} = {res}");
    let res = example.balanceOf(address_2).call().await?;
    println!("  Current balance of {address_2} = {res}");
    let res = example.ownerOf(token_id_3).call().await?;
    println!("  Owner of token {token_id_3} = {res}");

    println!("\n==============================");
    println!("  ApprovalForAll");
    println!("================================");

    // Mint another token to address_1 for testing
    let token_id_4 = U256::from(4);
    println!("Minting token {token_id_4} to {sender_2} for ApprovalForAll test");
    let pending_tx = example.mint(sender_2, token_id_4).send().await?;
    pending_tx.get_receipt().await?;

    println!("Setting approval for all tokens from {sender_2} to {sender}");
    let res = example.isApprovedForAll(sender_2, sender).call().await?;
    println!("{sender} approved for all tokens of {sender_2}: {res}");

    println!();

    println!("Executing setApprovalForAll...");
    let pending_tx = example_2.setApprovalForAll(sender, true).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("ApprovalForAll events");

    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        if let Ok(decoded_event) = Erc721::ApprovalForAll::decode_log(&primitive_log) {
            println!(
                "ApprovalForAll event log {} {:#?}",
                index + 1,
                decoded_event
            );
        }
    }

    println!();

    let res = example.isApprovedForAll(sender_2, sender).call().await?;
    println!(" {sender} approved for all tokens of {sender_2}: {res}");

    println!();

    println!("Testing transferFrom using ApprovalForAll:");
    let res = example.balanceOf(sender).call().await?;
    println!("  Current balance of {sender} = {res}");
    let res = example.balanceOf(sender_2).call().await?;
    println!("  Current balance of {sender_2} = {res}");
    println!(
        "  Transferring token {token_id_4} from {sender_2} to {sender} using operator approval"
    );
    let pending_tx = example
        .transferFrom(sender_2, sender, token_id_4)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Transfer events");
    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Erc721::Transfer::decode_log(&primitive_log)?;
        println!("Transfer event log {} {:#?}", index + 1, decoded_event);
    }

    let res = example.ownerOf(token_id_4).call().await?;
    println!("  Owner of token {token_id_4} after transfer = {res}");

    let res = example.balanceOf(sender).call().await?;
    println!("  Current balance of {sender} = {res}");
    let res = example.balanceOf(sender_2).call().await?;
    println!("  Current balance of {sender_2} = {res}");

    println!("Safe transferring token {token_id_4} from {sender} to {receiver_address}");
    let pending_tx = example
        .safeTransferFrom(sender, receiver_address, token_id_4, vec![])
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    println!("Safe transfer events");
    for (index, log) in receipt.logs().iter().enumerate() {
        let primitive_log: alloy::primitives::Log = log.clone().into();
        let decoded_event = Erc721::Transfer::decode_log(&primitive_log)?;
        println!("Safe transfer event log {} {:#?}", index + 1, decoded_event);
    }

    let res = example.balanceOf(sender).call().await?;
    println!("  Current balance of {sender} = {res}");
    let res = example.balanceOf(receiver_address).call().await?;
    println!("  Current balance of {receiver_address} = {res}");

    let res = example.ownerOf(token_id_4).call().await?;
    println!("  Owner of token {token_id_4} after safe transfer = {res}");

    let res = example.tokenUri(token_id_4).call().await?;
    println!("  Token URI of token {token_id_4} = {res}");

    let erc721_interface_id = FixedBytes::<4>::new([0x80, 0xac, 0x58, 0xcd]);
    let erc721_metadata_interface_id = FixedBytes::<4>::new([0x01, 0xff, 0xc9, 0xa7]);
    let res = example
        .supportsInterface(erc721_interface_id)
        .call()
        .await?;
    println!("  Supports IERC721 interface = {res}");
    let res = example
        .supportsInterface(erc721_metadata_interface_id)
        .call()
        .await?;
    println!("  Supports IERC721 metadata interface = {res}");

    Ok(())
}
