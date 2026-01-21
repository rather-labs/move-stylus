use alloy::primitives::{Bytes, FixedBytes, keccak256};
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

        function increment(bytes32 counter) external;
        function read(bytes32 counter) external returns (uint64);
        function setValue(bytes32 counter, uint64 value) external;

    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_COUNTER_WITH_INIT")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_COUNTER_WITH_INIT"))?;

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

    // Compute the constructor selector: keccak256("constructor()")[0..4]
    let constructor_selector = &keccak256("constructor()")[0..4];

    println!("==============================================================================");
    println!(" Calling constructor to create a new counter");
    println!("==============================================================================");
    // Call the constructor
    // The idea is that the constructor will be called upon deployment of the contract
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .input(Bytes::copy_from_slice(constructor_selector).into());
    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;

    let counter_id =
        FixedBytes::<32>::new(receipt.logs()[0].topics()[1].to_vec().try_into().unwrap());
    println!("✓ Counter created with ID: {counter_id:?}");

    println!("\nReading initial counter value (should be initialized to 25)");
    let res = example.read(counter_id).call().await?;
    println!("  Counter value: {res}");
    assert_eq!(
        res, 25u64,
        "Initial counter value should be 25 (from constructor)"
    );

    println!("\nIncrementing counter");
    let pending_tx = example.increment(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after increment");
    let res = example.read(counter_id).call().await?;
    println!("  Counter value: {res}");
    assert_eq!(res, 26u64, "Counter should be 26 after first increment");

    println!("\n==============================================================================");
    println!(" Testing constructor idempotency: calling constructor again");
    println!("==============================================================================");
    // Call it a second time to make sure the constructor is not called again
    let tx = TransactionRequest::default()
        .from(sender)
        .to(address)
        .input(Bytes::copy_from_slice(constructor_selector).into());
    let pending_tx = provider.send_transaction(tx).await?;
    let receipt = pending_tx.get_receipt().await?;

    // Check no log is emitted, meaning the constructor logic is not executed again
    assert_eq!(
        receipt.logs().len(),
        0,
        "Constructor should not emit logs when called again"
    );
    println!("✓ No logs emitted - constructor logic not executed again");

    // Read again and check the value has not changed
    let res = example.read(counter_id).call().await?;
    println!("  Counter value: {res}");
    assert_eq!(
        res, 26u64,
        "Counter value should remain 26 after redundant constructor call"
    );

    println!("\nIncrementing counter again");
    let pending_tx = example.increment(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after second increment");
    let res = example.read(counter_id).call().await?;
    println!("  Counter value: {res}");
    assert_eq!(res, 27u64, "Counter should be 27 after second increment");

    println!("\nSetting counter value to 42");
    let pending_tx = example.setValue(counter_id, 42).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after set");
    let res = example.read(counter_id).call().await?;
    println!("  Counter value: {res}");
    assert_eq!(res, 42u64, "Counter should be 42 after setValue");

    println!("\nIncrementing counter again");
    let pending_tx = example.increment(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after third increment");
    let res = example.read(counter_id).call().await?;
    println!("  Counter value: {res}");
    assert_eq!(res, 43u64, "Counter should be 43 after third increment");

    Ok(())
}
