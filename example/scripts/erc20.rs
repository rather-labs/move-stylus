use alloy::primitives::U256;
use alloy::primitives::address;
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
    contract Example {
        #[derive(Debug)]
        event Approval(address indexed owner, address indexed spender, uint256 value);

        #[derive(Debug)]
        event NewUID(bytes32 indexed uid);

        #[derive(Debug)]
        event Transfer(address indexed from, address indexed to, uint256 value);

        function constructor() public view;
        function mint(address to, uint256 amount) external view;
        function burn(address from, uint256 amount) external view;
        function balanceOf(address account) public view returns (uint256);
        function totalSupply() external view returns (uint256);
        function transfer(address recipient, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transferFrom(address sender, address recipient, uint256 amount) external returns (bool);
        function name() external view returns (string);
        function symbol() external view returns (string);
        function decimals() external view returns (uint8);
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let priv_key_2 =
        std::env::var("PRIV_KEY_2").map_err(|_| eyre!("No {} env var set", "PRIV_KEY_2"))?;

    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_ERC20")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_ERC20"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;
    let sender = signer.address();

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = Example::new(address, provider.clone());

    // Second sender
    let signer_2 = PrivateKeySigner::from_str(&priv_key_2)?;
    let sender_2 = signer_2.address();

    let provider_2 = Arc::new(
        ProviderBuilder::new()
            .wallet(signer_2)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let example_2 = Example::new(address, provider_2.clone());

    // Fund sender_2 with some ETH for gas
    let tx = TransactionRequest::default()
        .from(sender)
        .to(sender_2)
        .value(U256::from(1_000_000_000_000_000_000u128)); // 1 eth in wei
    let pending_tx = provider.send_transaction(tx).await?;
    pending_tx.get_receipt().await?;

    let address_3 = address!("0xcafecafecafecafecafecafecafecafecafecafe");

    // Helper to get last 8 hex chars of an address (0x + last 4 bytes)
    let short_addr = |addr: &Address| -> String {
        let s = format!("{addr}");
        format!("0x..{}", &s[s.len() - 8..])
    };

    let sender_short = short_addr(&sender);
    let sender_2_short = short_addr(&sender_2);
    let address_3_short = short_addr(&address_3);

    // ==================== Constructor ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║         Creating ERC20 Token         ║");
    println!("╚══════════════════════════════════════╝");

    let _pending_tx_ = example.constructor().send().await?;
    println!("✓ Contract initialized");

    // ==================== Contract Info ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║           Contract Info              ║");
    println!("╚══════════════════════════════════════╝");

    let res = example.totalSupply().call().await?;
    assert_eq!(res, U256::from(0));
    println!("  Total Supply:      {res}");

    let res = example.decimals().call().await?;
    assert_eq!(res, 18);
    println!("  Decimals:          {res}");

    let res = example.name().call().await?;
    println!("  Name:              {res}");

    let res = example.symbol().call().await?;
    println!("  Symbol:            {res}");

    // ==================== Mint ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║              Minting                 ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Initial balances:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_short}: {res} TST");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_2_short}: {res} TST");

    println!("\n  Minting 555555 TST to {sender_short}...");
    let pending_tx = example.mint(sender, U256::from(555555)).send().await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Mint complete");

    let res = example.totalSupply().call().await?;
    assert_eq!(res, U256::from(555555));
    println!("    Total Supply: {res}");

    println!("\n  Balances after minting:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(555555));
    println!("    {sender_short}: {res} TST");

    // ==================== Transfer ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║             Transfer                 ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Transferring 1000 TST: {sender_short} → {sender_2_short}");

    println!("\n  Balances before transfer:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(555555));
    println!("    {sender_short}: {res} TST");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {sender_2_short}: {res} TST");

    let pending_tx = example.transfer(sender_2, U256::from(1000)).send().await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Transfer complete");

    println!("\n  Balances after transfer:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(554555));
    println!("    {sender_short}: {res} TST");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(1000));
    println!("    {sender_2_short}: {res} TST");

    // ==================== Burn ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║              Burning                 ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Burning 11111 TST from {sender_short}...");
    let pending_tx = example.burn(sender, U256::from(11111)).send().await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Burn complete");

    let res = example.totalSupply().call().await?;
    assert_eq!(res, U256::from(544444));
    println!("    Total Supply: {res}");

    println!("\n  Balances after burning:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(543444));
    println!("    {sender_short}: {res} TST");

    // ==================== Allowance & TransferFrom ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║      Allowance & TransferFrom        ║");
    println!("╚══════════════════════════════════════╝");

    println!("\n  Before approval:");
    let res = example.allowance(sender_2, sender).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    Allowance ({sender_2_short} → {sender_short}): {res} TST");

    println!("\n  Approving {sender_short} to spend 100 TST from {sender_2_short}...");
    let pending_tx = example_2.approve(sender, U256::from(100)).send().await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ Approval granted");

    println!("\n  After approval:");
    let res = example.allowance(sender_2, sender).call().await?;
    assert_eq!(res, U256::from(100));
    println!("    Allowance ({sender_2_short} → {sender_short}): {res} TST");

    println!("\n  Balances before transferFrom:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(543444));
    println!("    {sender_short}: {res} TST");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(1000));
    println!("    {sender_2_short}: {res} TST");

    let res = example.balanceOf(address_3).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    {address_3_short}: {res} TST");

    println!("\n  Using transferFrom: {sender_2_short} → {address_3_short} (100 TST)");
    println!("    (spender: {sender_short})");
    let pending_tx = example
        .transferFrom(sender_2, address_3, U256::from(100))
        .send()
        .await?;
    pending_tx.get_receipt().await?;
    println!("  ✓ TransferFrom complete");

    println!("\n  After transferFrom:");
    let res = example.allowance(sender_2, sender).call().await?;
    assert_eq!(res, U256::from(0));
    println!("    Allowance ({sender_2_short} → {sender_short}): {res} TST (depleted)");

    println!("\n  Balances after transferFrom:");
    let res = example.balanceOf(sender).call().await?;
    assert_eq!(res, U256::from(543444));
    println!("    {sender_short}: {res} TST");

    let res = example.balanceOf(sender_2).call().await?;
    assert_eq!(res, U256::from(900));
    println!("    {sender_2_short}: {res} TST");

    let res = example.balanceOf(address_3).call().await?;
    assert_eq!(res, U256::from(100));
    println!("    {address_3_short}: {res} TST");

    // ==================== Done ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║        ✓ All tests passed!           ║");
    println!("╚══════════════════════════════════════╝\n");

    Ok(())
}
