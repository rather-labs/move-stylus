use alloy::primitives::U256;
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
        function create() external;
        function increment() external;
        function read() external returns (uint64);
        function setValue(uint64 value) external;
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_COUNTER_NAMED_ID")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_COUNTER_NAMED_ID"))?;

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

    println!("==============================================================================");
    println!(" Creating a new counter");
    println!("==============================================================================");
    let pending_tx = example.create().send().await?;
    let _receipt = pending_tx.get_receipt().await?;
    println!("✓ Counter created");

    println!("\nReading initial counter value");
    let res = example.read().call().await?;
    println!("  Counter value: {res}");
    assert_eq!(res, 25u64, "Initial counter value should be 25");

    println!("\nIncrementing counter");
    let pending_tx = example.increment().send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after increment");
    let res = example.read().call().await?;
    println!("  Counter value: {res}");
    assert_eq!(res, 26u64, "Counter should be 26 after first increment");

    println!("\nSetting counter value to 42");
    let pending_tx = example.setValue(42).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after set");
    let res = example.read().call().await?;
    println!("  Counter value: {res}");
    assert_eq!(res, 42u64, "Counter should be 42 after setValue");

    println!("\nIncrementing counter again");
    let pending_tx = example.increment().send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after second increment");
    let res = example.read().call().await?;
    println!("  Counter value: {res}");
    assert_eq!(res, 43u64, "Counter should be 43 after second increment");

    println!("\n==============================================================================");
    println!(" Testing access control: non-owner cannot modify counter");
    println!("==============================================================================");
    let priv_key_2 =
        std::env::var("PRIV_KEY_2").map_err(|_| eyre!("No {} env var set", "PRIV_KEY_2"))?;
    let signer_2 = PrivateKeySigner::from_str(&priv_key_2)?;
    let sender_2 = signer_2.address();

    let provider_2 = Arc::new(
        ProviderBuilder::new()
            .wallet(signer_2)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let example_2 = Example::new(address, provider_2.clone());

    println!("Funding {sender_2} with 5 ETH to pay for gas");
    let tx = TransactionRequest::default()
        .from(sender)
        .to(sender_2)
        .value(U256::from(5_000_000_000_000_000_000u128)); // 5 ETH in wei
    let pending_tx = provider.send_transaction(tx).await?;
    pending_tx.get_receipt().await?;
    println!("✓ Account funded");

    println!("\nAttempting to set counter value to 100 with non-owner account");
    let pending_tx = example_2.setValue(100).send().await;
    let tx_failed = pending_tx.is_err();
    println!("  Transaction failed: {tx_failed}");
    if let Err(e) = &pending_tx {
        println!("  Error: {e:?}");
    }
    assert!(tx_failed, "Non-owner should not be able to modify counter");

    println!("\nReading counter value to verify it was not changed");
    let res = example_2.read().call().await?;
    println!("  Counter value: {res}");
    assert_eq!(
        res, 43u64,
        "Counter should remain 43 after failed non-owner setValue attempt"
    );
    println!("✓ Access control verified: counter value unchanged");

    Ok(())
}
