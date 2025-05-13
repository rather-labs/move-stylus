//! Example on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use dotenv::dotenv;
use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
};
use eyre::eyre;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS"))?;
    abigen!(
        Example,
        r#"[
            function echo(uint128 x) external view returns (uint128)
            function getCopiedLocal() external view returns (uint128)
            function echoSignerWithInt(uint8 y) public view returns (uint8, address)
        ]"#
    );

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let address: Address = contract_address.parse()?;

    let wallet = LocalWallet::from_str(&priv_key)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    let example = Example::new(address, client.clone());

    let num = example.echo(123).call().await;
    println!("Example echo = {:?}", num);

    let num = example.get_copied_local().call().await;
    println!("Example getCopiedLocal = {:?}", num);

    // TODO: Common calls do not have a signer, but if we a function with a signer, it returns an
    // address that is probably things in memory. This could be a security issue and must be taken
    // care of.
    let num = example.echo_signer_with_int(42).call().await;
    println!("Example echoSignerWithInt = {:?}", num);

    // TODO: To verify functions called with a transaction, we need to support logging functions in
    // the host (https://github.com/OffchainLabs/stylus-sdk-rs/blob/1801df8872638ceb3d7db074e21e605fc6e68350/stylus-sdk/src/hostio.rs#L404-L426)
    // Leave this code here as an example on how to call a function with a tx
    /*
    use ethers::{
        types::{TransactionRequest, H160, U256},
        utils::parse_ether
    }

    let data = example.echo_signer_with_int(42).calldata().unwrap();
    let tx = TransactionRequest::new()
        .to(H160::from_str(&contract_address).unwrap())
        .value(parse_ether(0.01)?)
        .data(data);

    // send it!
    let pending_tx = client.send_transaction(tx, None).await?;

    // get the mined tx
    let receipt = pending_tx
        .await?
        .ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;
    let tx = client.get_transaction(receipt.transaction_hash).await?;

    println!("Sent tx: {tx:?}\n");
    println!("Tx receipt: {receipt:?}");
    */

    Ok(())
}
