//! on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use alloy_sol_types::{SolCall, SolType, sol};
use dotenv::dotenv;
use ethers::{
    contract::abigen, middleware::SignerMiddleware, providers::{Http, Middleware, Provider}, signers::{LocalWallet, Signer}, types::{
        transaction::eip2718::TypedTransaction, Address, TransactionRequest, H160
    }, utils::parse_ether
};

use eyre::eyre;
use std::str::FromStr;
use std::sync::Arc;



/*
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
*/

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS"))?;

    abigen!(
        Example,
        r#"
        [
            function echo(uint128 x) external view returns (uint128)
            function getCopiedLocal() external view returns (uint128)
            function getConstant() external view returns (uint128)
            function getConstantLocal() external view returns (uint128)
            function getLocal(uint128 x) external view returns (uint128)
            function echoSignerWithInt(uint8 y) public view returns (uint8, address)
            function txContextProperties() public view returns (address, uint256, uint64, uint256, uint64, uint64, uint64, uint256)
            function fibonacci(uint64 n) external view returns (uint64)
            function sumSpecial(uint64 n) external view returns (uint64)
        ]
        "#
    );

    sol!(
        #[allow(missing_docs)]

        #[derive(Debug)]
        struct Bar {
            uint32 n;
            uint128 o;
        }

        #[derive(Debug)]
        struct Foo {
            uint16 g;
            Bar p;
            address q;
            uint32[] r;
            // uint128[] s;
            bool t;
            uint8 u;
            uint16 v;
            uint32 w;
            uint64 x;
            uint128 y;
            uint256 z;
        }

        function createFooU16(uint16 a, uint16 b) external view returns (Foo);
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


    let num = example.echo(123).call().await?;
    println!("echo(123) = {}", num);

    let num = example.get_constant().call().await?;
    println!("getConstant = {}", num);

    let num = example.get_constant_local().call().await?;
    println!("getConstantLocal = {}", num);

    let num = example.get_copied_local().call().await?;
    println!("getCopiedLocal = {}", num);

    let num = example.get_local(456).call().await?;
    println!("getLocal = {}", num);

    let tx_context = example.tx_context_properties().call().await?;
    println!("txContextProperties:");
    println!("  - msg.sender: {:?}", tx_context.0);
    println!("  - msg.value: {}", tx_context.1);
    println!("  - block.number: {}", tx_context.2);
    println!("  - block.basefee: {}", tx_context.3);
    println!("  - block.gas_limit: {}", tx_context.4);
    println!("  - block.timestamp: {}", tx_context.5);
    println!("  - chainid: {}", tx_context.6);
    println!("  - tx.gas_price: {}", tx_context.7);

    let fib10 = example.fibonacci(10).call().await?;
    println!("fibonacci(10) = {}", fib10);

    let fib20 = example.fibonacci(20).call().await?;
    println!("fibonacci(20) = {}", fib20);

    let sum_special_2 = example.sum_special(2).call().await?;
    println!("sumSpecial(2) = {}", sum_special_2);

    let sum_special_4 = example.sum_special(4).call().await?;
    println!("sumSpecial(4) = {}", sum_special_4);

    // This simple call will inject the "from" field as asigner
    let ret = example.echo_signer_with_int(42).call().await;
    println!("echoSignerWithInt = {:?}", ret);

    // A real tx should write in the logs the signer's address
    // 0x3f1eae7d46d88f08fc2f8ed27fcb2ab183eb2d0e
    let data = example.echo_signer_with_int(43).calldata().unwrap();

    call_contract::<createFooU16Call, sol!((Foo,))>(
        client.clone(),
        address,
        createFooU16Call { a: 1, b: 2 }
    ).await;

    let tx = TransactionRequest::new()
        .to(H160::from_str(&contract_address).unwrap())
        .value(parse_ether(0.1)?)
        .data(data);

    let pending_tx = client.send_transaction(tx, None).await?;
    let receipt = pending_tx
        .await?
        .ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;
    println!(
        "echoSignerWithInt - transaction log data: {:?}",
        receipt.logs.first().map(|l| &l.data)
    );

    Ok(())
}

async fn call_contract<T: SolCall, R: SolType>(
    client: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    address: H160,
    call_data: T,
) where <R as alloy_sol_types::SolType>::RustType: std::fmt::Debug {
    let response = client.call(
        &TypedTransaction::Legacy(
            TransactionRequest {
                to: Some(address.into()),
                data: Some(call_data.abi_encode().into()),
                ..Default::default()
            }),
        None)
        .await
        .unwrap();

    println!("{}: {:?}", std::any::type_name::<T>(), response.as_ref());
    println!("{}: {:?}", std::any::type_name::<T>(), R::abi_decode(response.as_ref()));
}
