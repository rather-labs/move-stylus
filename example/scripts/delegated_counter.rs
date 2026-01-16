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
    contract Example {
        function changeLogic(bytes32 counter, address logic_address) external;
        function create(address contract_logic) external;
        function increment(bytes32 counter) external;
        function incrementModifyAfter(bytes32 counter) external;
        function incrementModifyBefore(bytes32 counter) external;
        function incrementModifyBeforeAfter(bytes32 counter) external;
        function logicAddress(bytes32 counter) external returns (address);
        function read(bytes32 counter) external returns (uint64);
        function setValue(bytes32 counter, uint64 value) external;
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_DELEGATED_COUNTER")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_DELEGATED_COUNTER"))?;

    let contract_address_logic_1 = std::env::var("CONTRACT_ADDRESS_DELEGATED_COUNTER_LOGIC_1")
        .map_err(|_| {
            eyre!(
                "No {} env var set",
                "CONTRACT_ADDRESS_DELEGATED_COUNTER_LOGIC_1"
            )
        })?;

    let contract_address_logic_2 = std::env::var("CONTRACT_ADDRESS_DELEGATED_COUNTER_LOGIC_2")
        .map_err(|_| {
            eyre!(
                "No {} env var set",
                "CONTRACT_ADDRESS_DELEGATED_COUNTER_LOGIC_2"
            )
        })?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;
    let sender = signer.address();

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let address_logic_1 = Address::from_str(&contract_address_logic_1)?;
    let address_logic_2 = Address::from_str(&contract_address_logic_2)?;
    let example = Example::new(address, provider.clone());

    println!("==============================================================================");
    println!(" Creating a new counter with logic contract 1");
    println!("==============================================================================");
    let pending_tx = example.create(address_logic_1).send().await?;
    let receipt = pending_tx.get_receipt().await?;

    let counter_id =
        FixedBytes::<32>::new(receipt.logs()[0].topics()[1].to_vec().try_into().unwrap());
    println!("✓ Counter created with ID: {counter_id:?}");

    println!("\nReading contract logic address");
    let res = example.logicAddress(counter_id).call().await?;
    println!("  Logic address: {res}");
    assert_eq!(
        res, address_logic_1,
        "Logic address should match logic contract 1"
    );

    println!("\n==============================================================================");
    println!(" Testing basic operations with logic contract 1");
    println!("==============================================================================");

    println!("\nReading initial counter value");
    let res = example.read(counter_id).call().await?;
    let mut counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(counter_value, 25u64, "Initial counter value should be 25");

    println!("\nIncrementing counter");
    let pending_tx = example.increment(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after increment");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 26u64,
        "Counter should be 26 after first increment"
    );

    println!("\nSetting counter value to 42");
    let pending_tx = example.setValue(counter_id, 42).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after set");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(counter_value, 42u64, "Counter should be 42 after setValue");

    println!("\nIncrementing counter again");
    let pending_tx = example.increment(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after second increment");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 43u64,
        "Counter should be 43 after second increment"
    );

    println!("\nTesting incrementModifyBefore (should increment by 10 and 1)");
    let pending_tx = example.incrementModifyBefore(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after incrementModifyBefore");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 54u64,
        "Counter should be 54 (43 + 10 + 1) after incrementModifyBefore"
    );

    println!("\nTesting incrementModifyAfter (should increment by 20 and 1)");
    let pending_tx = example.incrementModifyAfter(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after incrementModifyAfter");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 75u64,
        "Counter should be 75 (54 + 20 + 1) after incrementModifyAfter"
    );

    println!("\nTesting incrementModifyBeforeAfter (should increment by 10, 1, and 20)");
    let pending_tx = example
        .incrementModifyBeforeAfter(counter_id)
        .send()
        .await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after incrementModifyBeforeAfter");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 106u64,
        "Counter should be 106 (75 + 10 + 1 + 20) after incrementModifyBeforeAfter"
    );

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

    println!("Funding {sender_2} with 1 ETH to pay for gas");
    let tx = TransactionRequest::default()
        .from(sender)
        .to(sender_2)
        .value(U256::from(1_000_000_000_000_000_000u128)); // 1 ETH in wei
    let pending_tx = provider.send_transaction(tx).await?;
    pending_tx.get_receipt().await?;
    println!("✓ Account funded");

    println!("\nAttempting to set counter value to 100 with non-owner account");
    let pending_tx = example_2.setValue(counter_id, 100).send().await;
    let tx_failed = pending_tx.is_err();
    println!("  Transaction failed: {tx_failed}");
    assert!(tx_failed, "Non-owner should not be able to modify counter");

    println!("\nReading counter value to verify it was not changed");
    let res = example_2.read(counter_id).call().await?;
    println!("  Counter value: {res}");
    assert_eq!(
        res, counter_value,
        "Counter should remain unchanged after failed non-owner setValue attempt"
    );
    println!("✓ Access control verified: counter value unchanged");

    println!("\n==============================================================================");
    println!(" Changing contract logic from {address_logic_1}");
    println!(" to {address_logic_2}");
    println!("==============================================================================");
    let pending_tx = example
        .changeLogic(counter_id, address_logic_2)
        .send()
        .await?;
    let _receipt = pending_tx.get_receipt().await?;
    println!("✓ Logic contract changed");

    println!("\nVerifying logic address");
    let res = example.logicAddress(counter_id).call().await?;
    println!("  Logic address: {res}");
    assert_eq!(
        res, address_logic_2,
        "Logic address should match logic contract 2"
    );

    println!("\n==============================================================================");
    println!(" Testing operations with logic contract 2 (different behavior)");
    println!("==============================================================================");

    println!("\nReading counter value before operations");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");

    println!("\nIncrementing counter (logic 2 increments by 2)");
    let pending_tx = example.increment(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after increment");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 108u64,
        "Counter should be 108 (106 + 2) after increment with logic 2"
    );

    println!("\nSetting counter value to 42 (logic 2 doubles it, so should set to 84)");
    let pending_tx = example.setValue(counter_id, 42).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after set");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 84u64,
        "Counter should be 84 (42 * 2) after setValue with logic 2"
    );

    println!("\nIncrementing counter again");
    let pending_tx = example.increment(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after second increment");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 86u64,
        "Counter should be 86 (84 + 2) after second increment with logic 2"
    );

    println!("\nTesting incrementModifyBefore with logic 2 (should increment by 10 and 2)");
    let pending_tx = example.incrementModifyBefore(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after incrementModifyBefore");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 98u64,
        "Counter should be 98 (86 + 10 + 2) after incrementModifyBefore with logic 2"
    );

    println!("\nTesting incrementModifyAfter with logic 2 (should increment by 20 and 2)");
    let pending_tx = example.incrementModifyAfter(counter_id).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after incrementModifyAfter");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 120u64,
        "Counter should be 120 (98 + 20 + 2) after incrementModifyAfter with logic 2"
    );

    println!(
        "\nTesting incrementModifyBeforeAfter with logic 2 (should increment by 10, 2, and 20)"
    );
    let pending_tx = example
        .incrementModifyBeforeAfter(counter_id)
        .send()
        .await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("Reading counter value after incrementModifyBeforeAfter");
    let res = example.read(counter_id).call().await?;
    counter_value = res;
    println!("  Counter value: {counter_value}");
    assert_eq!(
        counter_value, 152u64,
        "Counter should be 152 (120 + 10 + 2 + 20) after incrementModifyBeforeAfter with logic 2"
    );

    println!("\n==============================================================================");
    println!(" Testing access control again: non-owner cannot modify counter");
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

    println!("Funding {sender_2} with 1 ETH to pay for gas");
    let tx = TransactionRequest::default()
        .from(sender)
        .to(sender_2)
        .value(U256::from(1_000_000_000_000_000_000u128)); // 1 ETH in wei
    let pending_tx = provider.send_transaction(tx).await?;
    pending_tx.get_receipt().await?;
    println!("✓ Account funded");

    println!("\nAttempting to set counter value to 100 with non-owner account");
    let pending_tx = example_2.setValue(counter_id, 100).send().await;
    let tx_failed = pending_tx.is_err();
    println!("  Transaction failed: {tx_failed}");
    assert!(tx_failed, "Non-owner should not be able to modify counter");

    println!("\nReading counter value to verify it was not changed");
    let res = example_2.read(counter_id).call().await?;
    println!("  Counter value: {res}");
    assert_eq!(
        res, counter_value,
        "Counter should remain unchanged after failed non-owner setValue attempt"
    );
    println!("✓ Access control verified: counter value unchanged");

    Ok(())
}
