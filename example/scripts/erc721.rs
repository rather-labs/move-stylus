use alloy::primitives::{Bytes, address, keccak256};
use alloy::primitives::{FixedBytes, U256};
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
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
        event Approval(address indexed owner, address indexed approved, uint256 indexed token_id);

        #[derive(Debug)]
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

        #[derive(Debug)]
        event NewUID(bytes32 indexed uid);

        #[derive(Debug)]
        event Transfer(address indexed from, address indexed to, uint256 indexed token_id);

        function approve(address to, uint256 token_id) external;
        function balanceOf(address owner) external returns (uint256);
        function burn(uint256 token_id) external;
        function getApproved(uint256 token_id) external returns (address);
        function isApprovedForAll(address owner, address operator) external returns (bool);
        function mint(address to, uint256 token_id) external;
        function name() external returns (string);
        function ownerOf(uint256 token_id) external returns (address);
        function safeTransferFrom(address from, address to, uint256 token_id, uint8[] data) external;
        function setApprovalForAll(address operator, bool approved) external;
        function supportsInterface(bytes4 interface_id) external returns (bool);
        function symbol() external returns (string);
        function tokenURI(uint256 token_id) external returns (string);
        function totalSupply() external returns (uint256);
        function transfer(address from, address to, uint256 token_id) external;
        function transferFrom(address from, address to, uint256 token_id) external;
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

    let tx = TransactionRequest::default()
        .from(sender)
        .to(sender_2)
        .value(U256::from(1_000_000_000_000_000_000u128)); // 1 eth in wei
    let pending_tx = provider.send_transaction(tx).await?;
    pending_tx.get_receipt().await?;

    let address_2 = address!("0xcafecafecafecafecafecafecafecafecafecafe");

    // Helper to get last 8 hex chars of an address (0x + last 4 bytes)
    let short_addr = |addr: &Address| -> String {
        let s = format!("{addr}");
        format!("0x..{}", &s[s.len() - 8..])
    };

    let sender_short = short_addr(&sender);
    let sender_2_short = short_addr(&sender_2);
    let address_2_short = short_addr(&address_2);
    let receiver_short = short_addr(&receiver_address);

    // ==================== Constructor ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║         Creating ERC721 Token        ║");
    println!("╚══════════════════════════════════════╝");

    // Compute the constructor selector: keccak256("constructor()")[0..4]
    let constructor_selector = &keccak256("constructor()")[0..4];
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .input(Bytes::copy_from_slice(constructor_selector).into());
    let _pending_tx_ = provider.send_transaction(tx).await?;
    println!("✓ Contract initialized");

    // ==================== Contract Info ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║           Contract Info              ║");
    println!("╚══════════════════════════════════════╝");

    let res = example.totalSupply().call().await?;
    assert_eq!(res, U256::from(0));
    println!("  Total Supply:      {res}");

    let res = example.name().call().await?;
    println!("  Name:              {res}");

    let res = example.symbol().call().await?;
    println!("  Symbol:            {res}");

    let erc721_interface_id = FixedBytes::<4>::new([0x80, 0xac, 0x58, 0xcd]);
    let erc721_metadata_interface_id = FixedBytes::<4>::new([0x01, 0xff, 0xc9, 0xa7]);

    let res = example
        .supportsInterface(erc721_interface_id)
        .call()
        .await?;
    assert!(res);
    println!(" IERC721 supported: {res}");

    let res = example
        .supportsInterface(erc721_metadata_interface_id)
        .call()
        .await?;
    assert!(res);
    println!(" IERC721Metadata supported: {res}");

    let res = example.tokenURI(U256::from(12345)).call().await?;
    assert_eq!(res, "https://examplerc721.com/token/12345");
    println!(" Token URI: {res}");
    // ==================== Mint ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║              Minting                 ║");
    println!("╚══════════════════════════════════════╝");

    let token_id_1 = U256::from(1);
    let token_id_2 = U256::from(2);
    let token_id_3 = U256::from(3);

    println!("\n  Initial balances:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_short}: {res}");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_2_short}: {res}");

    println!("\n  Minting tokens...");
    println!("    → Token #{token_id_1} to {sender_short}");
    let pending_tx = example.mint(sender, token_id_1).send().await?;
    pending_tx.get_receipt().await?;

    println!("    → Token #{token_id_2} to {sender_short}");
    let pending_tx = example.mint(sender, token_id_2).send().await?;
    pending_tx.get_receipt().await?;

    println!("    → Token #{token_id_3} to {sender_2_short}");
    let pending_tx = example.mint(sender_2, token_id_3).send().await?;
    pending_tx.get_receipt().await?;

    let res = example.totalSupply().call().await?;
    assert_eq!(res, U256::from(3));
    println!("    Total Supply: {res}");

    println!("\n  Balances after minting:");

    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(2));
    println!("    {sender_short}: {res}");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(1));
    println!("    {sender_2_short}: {res}");

    println!("\n  Ownership:");
    let res = example.ownerOf(token_id_1).call().await?;
    assert_eq!(res, sender);
    println!("    Token #{token_id_1}: {sender_short}");

    let res = example.ownerOf(token_id_2).call().await?;
    assert_eq!(res, sender);
    println!("    Token #{token_id_2}: {sender_short}");

    let res = example.ownerOf(token_id_3).call().await?;
    assert_eq!(res, sender_2);
    println!("    Token #{token_id_3}: {sender_2_short}");

    // ==================== Burn ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║              Burning                 ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Burning token #{token_id_2}...");
    let pending_tx = example.burn(token_id_2).send().await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Token burned");
    let res = example.totalSupply().call().await?;
    assert_eq!(res, U256::from(2));
    println!("    Total Supply: {res}");

    println!("\n  Balances after burning:");

    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(1));
    println!("    {sender_short}: {res}");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(1));
    println!("    {sender_2_short}: {res}");

    // ==================== Transfer ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║             Transfer                 ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Transferring token #{token_id_1}: {sender_short} → {sender_2_short}");
    let pending_tx = example
        .transfer(sender, sender_2, token_id_1)
        .send()
        .await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Transfer complete");
    let res = example.ownerOf(token_id_1).call().await?;
    assert_eq!(res, sender_2);
    println!("    Token #{token_id_1} new owner: {sender_2_short}");

    println!("\n  Balances after transfer:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_short}: {res}");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(2));
    println!("    {sender_2_short}: {res}");

    // ==================== Approval ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║        Approval & TransferFrom       ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Before approval:");
    let res = example.getApproved(token_id_3).call().await?;
    assert_eq!(res, Address::ZERO);
    println!("    Approved for #{token_id_3}: (none)");

    println!("\n  Approving {sender_short} to transfer #{token_id_3}...");
    let pending_tx = example_2.approve(sender, token_id_3).send().await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Approval granted");

    println!("\n  After approval:");
    let res = example.getApproved(token_id_3).call().await?;
    assert_eq!(res, sender);
    println!("    Approved for #{token_id_3}: {sender_short}");

    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_short}: {res}");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(2));
    println!("    {sender_2_short}: {res}");

    println!("\n  Using transferFrom: {sender_2_short} → {address_2_short} (token #{token_id_3})");
    let pending_tx = example
        .transferFrom(sender_2, address_2, token_id_3)
        .send()
        .await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ TransferFrom complete");
    let res = example.getApproved(token_id_3).call().await?;
    assert_eq!(res, Address::ZERO);
    println!("    Approved for #{token_id_3}: (cleared)");
    let res = example.ownerOf(token_id_3).call().await?;
    assert_eq!(res, address_2);
    println!("    Token #{token_id_3} owner: {address_2_short}");

    println!("\n  Balances after transferFrom:");

    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_short}: {res}");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(1));
    println!("    {sender_2_short}: {res}");

    let res = example.balanceOf(address_2).call().await?;
    assert_eq!(res, U256::from(1));
    println!("    {address_2_short}: {res}");

    // ==================== ApprovalForAll ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║           ApprovalForAll             ║");
    println!("╚══════════════════════════════════════╝");

    let token_id_4 = U256::from(4);
    println!("\n  Minting token #{token_id_4} to {sender_2_short}...");
    let pending_tx = example.mint(sender_2, token_id_4).send().await?;
    pending_tx.get_receipt().await?;

    println!("\n  Before setApprovalForAll:");
    let res = example.isApprovedForAll(sender_2, sender).call().await?;
    assert!(!res);
    println!("    {sender_short} is operator for {sender_2_short}: {res}");

    println!("\n  Setting {sender_short} as operator for {sender_2_short}...");
    let pending_tx = example_2.setApprovalForAll(sender, true).send().await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Operator approval granted");

    println!("\n  After setApprovalForAll:");
    let res = example.isApprovedForAll(sender_2, sender).call().await?;
    assert!(res);
    println!("    {sender_short} is operator for {sender_2_short}: {res}");

    println!("\n  Balances before operator transfer:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_short}: {res}");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(2));
    println!("    {sender_2_short}: {res}");

    println!(
        "\n  Using transferFrom (operator): {sender_2_short} → {sender_short} (token #{token_id_4})"
    );
    let pending_tx = example
        .transferFrom(sender_2, sender, token_id_4)
        .send()
        .await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Operator transfer complete");
    let res = example.ownerOf(token_id_4).call().await?;
    assert_eq!(res, sender);
    println!("    Token #{token_id_4} owner: {sender_short}");

    println!("\n  Balances after operator transfer:");

    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(1));
    println!("    {sender_short}: {res}");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(1));
    println!("    {sender_2_short}: {res}");

    // ==================== Safe Transfer ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║           Safe Transfer              ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Safe transferring token #{token_id_4}: {sender_short} → {receiver_short}");
    let pending_tx = example
        .safeTransferFrom(sender, receiver_address, token_id_4, vec![])
        .send()
        .await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Safe transfer complete");
    let res = example.ownerOf(token_id_4).call().await?;
    assert_eq!(res, receiver_address);
    println!("    Token #{token_id_4} owner: {receiver_short}");

    println!("\n  Balances after safe transfer:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_short}: {res}");

    let res = example.balanceOf(receiver_address).call().await?;
    assert_eq!(res, U256::from(1));
    println!("    {receiver_short}: {res}");

    // ==================== Done ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║        ✓ All tests passed!           ║");
    println!("╚══════════════════════════════════════╝\n");

    Ok(())
}
