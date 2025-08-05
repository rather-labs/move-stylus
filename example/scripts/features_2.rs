//! Example on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use alloy::signers::local::PrivateKeySigner;
use alloy::{primitives::{Address, address}, providers::ProviderBuilder, sol, transports::http::reqwest::Url};
use dotenv::dotenv;
use eyre::eyre;
use std::str::FromStr;
use std::sync::Arc;

sol!(
    #[sol(rpc)]
    #[allow(missing_docs)]
    contract Example {
        function echoWithGenericFunctionU16(uint16 x) external view returns (uint16);
        function echoWithGenericFunctionVec32(uint32[] x) external view returns (uint32[]);
        function echoWithGenericFunctionU16Vec32(uint16 x, uint32[] y) external view returns (uint16, uint32[]);
        function echoWithGenericFunctionAddressVec128(address x, uint128[] y) external view returns (address, uint128[]);
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;

    let contract_address = std::env::var("CONTRACT_ADDRESS_FEATURES_2")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_FEATURES_2"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = Example::new(address, provider.clone());

    let ret = example.echoWithGenericFunctionU16(42).call().await?;
    println!("echoWithGenericFunctionU16 {ret}");

    let ret = example.echoWithGenericFunctionVec32(vec![1,2,3]).call().await?;
    println!("echoWithGenericFunctionVec32 {ret:?}");

    let ret = example.echoWithGenericFunctionU16Vec32(42, vec![4,5,6]).call().await?;
    println!("echoWithGenericFunctionU16Vec32 ({}, {:?})", ret._0, ret._1);

    let ret = example.echoWithGenericFunctionAddressVec128(address!("0x1234567890abcdef1234567890abcdef12345678"), vec![7,8,9]).call().await?;
    println!("echoWithGenericFunctionAddressVec256 ({}, {:?})", ret._0, ret._1);

    Ok(())
}
