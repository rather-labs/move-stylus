use alloy::hex;
use alloy::primitives::U256;
use alloy::providers::Provider;
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use alloy::{primitives::Address, providers::ProviderBuilder, sol, transports::http::reqwest::Url};
use dotenv::dotenv;
use eyre::eyre;
use std::io::Read;
use std::str::FromStr;
use std::sync::Arc;

sol!(
    #[sol(rpc)]
    #[allow(missing_docs)]
    contract Example {
        function create(address contract_logic) public view;
        function read() public view returns (uint64);
        function logicAddress() public view returns (address);
        function changeLogic(address logic_address) public view;
        function incrementModifyBefore() public view;
        function incrementModifyAfter() public view;
        function incrementModifyBeforeAfter() public view;
        function increment() public view;
        function setValue(uint64 value) public view;
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address =
        std::env::var("CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID").map_err(|_| {
            eyre!(
                "No {} env var set",
                "CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID"
            )
        })?;

    let contract_address_logic_1 =
        std::env::var("CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID_LOGIC_1").map_err(|_| {
            eyre!(
                "No {} env var set",
                "CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID_LOGIC_1"
            )
        })?;

    let contract_address_logic_2 =
        std::env::var("CONTRACT_ADDRESS_DELEGATED_COUNTER_NAMED_ID_LOGIC_2").map_err(|_| {
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

    let pending_tx = example.create(address_logic_1).send().await?;
    let receipt = pending_tx.get_receipt().await?;

    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("create tx 0x{}", hex::encode(&raw));
    }

    println!("\nReading contract logic address");
    let res = example.logicAddress().call().await?;
    println!("counter = {}", res);

    println!("==============================================================================");
    println!(" Executing increment and setValue on logic contract:");
    println!(" {address_logic_1}");
    println!("==============================================================================");

    println!("\nReading value before increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment tx");

    let pending_tx = example.increment().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: {:?}", raw.bytes());
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSetting counter to number 42");
    let pending_tx = example.setValue(42).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading counter after set");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment tx");
    let pending_tx = example.increment().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment BEFORE tx (shuld increment by 10 and 1)");
    let pending_tx = example.incrementModifyBefore().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment AFTER tx (shuld increment by 20 and 1)");
    let pending_tx = example.incrementModifyAfter().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment BEFORE And AFTER tx (shuld increment by 10, 1, and 20)");
    let pending_tx = example.incrementModifyBeforeAfter().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    // Add a new sender and try to set the value
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

    println!("\nFunding {sender_2} with some ETH to pay for the gas");
    let tx = TransactionRequest::default()
        .from(sender)
        .to(sender_2)
        .value(U256::from(1_000_000_000_000_000_000u128)); // 5 eth in wei
    let pending_tx = provider.send_transaction(tx).await?;
    pending_tx.get_receipt().await?;

    println!("\nSending set value to 100 tx with the account that is not the owner");
    let pending_tx = example_2.setValue(100).send().await;
    println!("Tx failed?: {:?}", pending_tx.is_err());

    // Value did not change as the sender is not the owner
    println!("\nReading value after set value");
    let res = example_2.read().call().await?;
    println!("counter = {}", res);

    println!("==============================================================================");
    println!(" Changing contract logic from {address_logic_1}");
    println!(" to {address_logic_2}");
    println!("==============================================================================\n");
    let pending_tx = example.changeLogic(address_logic_2).send().await?;
    let _receipt = pending_tx.get_receipt().await?;

    println!("==============================================================================");
    println!(" Executing increment and setValue on logic contract:");
    println!(" {address_logic_2}");
    println!("==============================================================================");

    println!("\nReading value before increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment tx");

    let pending_tx = example.increment().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSetting counter to number 42 (should set 42*2)");
    let pending_tx = example.setValue(42).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading counter after set");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment tx");
    let pending_tx = example.increment().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment BEFORE tx (shuld increment by 10 and 2)");
    let pending_tx = example.incrementModifyBefore().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment AFTER tx (shuld increment by 20 and 2)");
    let pending_tx = example.incrementModifyAfter().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    println!("\nSending increment BEFORE And AFTER tx (shuld increment by 10, 2, and 20)");
    let pending_tx = example.incrementModifyBeforeAfter().send().await?;
    let receipt = pending_tx.get_receipt().await?;
    for log in receipt.logs() {
        let raw = log.data().data.0.clone();
        println!("increment logs 0: 0x{}", hex::encode(raw));
    }

    println!("\nReading value after increment");
    let res = example.read().call().await?;
    println!("counter = {}", res);

    // Add a new sender and try to set the value
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

    println!("\nFunding {sender_2} with some ETH to pay for the gas");
    let tx = TransactionRequest::default()
        .from(sender)
        .to(sender_2)
        .value(U256::from(1_000_000_000_000_000_000u128)); // 5 eth in wei
    let pending_tx = provider.send_transaction(tx).await?;
    pending_tx.get_receipt().await?;

    println!("\nSending set value to 100 tx with the account that is not the owner");
    let pending_tx = example_2.setValue(100).send().await;
    println!("Tx failed?: {:?}", pending_tx.is_err());

    // Value did not change as the sender is not the owner
    println!("\nReading value after set value");
    let res = example_2.read().call().await?;
    println!("counter = {}", res);

    Ok(())
}
