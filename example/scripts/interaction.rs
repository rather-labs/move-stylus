//! Example on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use dotenv::dotenv;
use ethers::{
    abi::{ParamType, decode},
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{
        Address, H160, NameOrAddress, TransactionRequest, transaction::eip2718::TypedTransaction,
    },
    utils::parse_ether,
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
            function getConstant() external view returns (uint128)
            function getConstantLocal() external view returns (uint128)
            function echoSignerWithInt(uint8 y) public view returns (uint8, address)
            function sum32(uint32 x, uint32 y) public view returns (uint32)
            function sum128(uint128 x, uint128 y) public view returns (uint128)
            function sub32(uint32 x, uint32 y) public view returns (uint32)
            function sub128(uint128 x, uint128 y) public view returns (uint128)
            function mul32(uint32 x, uint32 y) public view returns (uint32)
            function mul128(uint128 x, uint128 y) public view returns (uint128)
            function div32(uint32 x, uint32 y) public view returns (uint32)
            function div128(uint128 x, uint128 y) public view returns (uint128)
            function mod32(uint32 x, uint32 y) public view returns (uint32)
            function mod128(uint128 x, uint128 y) public view returns (uint128)
        ]"#
    );

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let address: Address = contract_address.parse()?;

    let wallet = LocalWallet::from_str(&priv_key)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider.clone(),
        wallet.clone().with_chain_id(chain_id),
    ));

    let example = Example::new(address, client.clone());

    let num = example.echo(123).call().await;
    println!("Example echo = {:?}", num);

    let num = example.get_copied_local().call().await;
    println!("Example getCopiedLocal = {:?}", num);

    // This simple call will inject the "from" field as asigner
    let ret = example.echo_signer_with_int(42).call().await;
    println!("Example echoSignerWithInt = {:?}", ret);

    let ret = example.sum_32(u16::MAX as u32, u16::MAX as u32).call().await;
    println!("Example sum8 = {:?}", ret);

    let ret = example.sum_128(u64::MAX as u128, u64::MAX as u128).call().await;
    println!("Example sum64 = {:?}", ret);

    // Removing the "from" field should return set the signer address as 0x0
    let data = example.echo_signer_with_int(43).calldata().unwrap();
    let ret = decode(
        &[ParamType::Uint(8), ParamType::Address],
        provider
            .call_raw(&TypedTransaction::Legacy(TransactionRequest {
                from: None,
                to: Some(NameOrAddress::Address(
                    H160::from_str(&contract_address).unwrap(),
                )),
                data: Some(data),
                ..Default::default()
            }))
            .await
            .unwrap()
            .as_ref(),
    )
    .unwrap();
    println!("Example echoSignerWithInt = {:?}", ret);

    // A real tx should write in the logs the signer's address
    // 0x3f1eae7d46d88f08fc2f8ed27fcb2ab183eb2d0e
    let data = example.echo_signer_with_int(42).calldata().unwrap();

    let tx = TransactionRequest::new()
        .to(H160::from_str(&contract_address).unwrap())
        .value(parse_ether(0.1)?)
        .data(data);

    let pending_tx = client.send_transaction(tx, None).await?;
    let receipt = pending_tx
        .await?
        .ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;
    println!(
        "Example echoSignerWithInt - transaction log data: {:?}",
        receipt.logs.first().map(|l| &l.data)
    );

    Ok(())
}
