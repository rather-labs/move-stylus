//! Example on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.
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
        struct Bar {
            uint32 a;
            uint128 b;
        }

        #[derive(Debug)]
        struct Foo {
            uint16 c;
            Bar d;
            address e;
            bool f;
            uint64 g;
            uint256 h;
            uint32[] i;
        }

        #[derive(Debug)]
        struct Baz {
            uint16 c;
            Bar d;
            address e;
            bool f;
            uint64 g;
            uint256 h;
        }

        #[derive(Debug)]
        enum TestEnum {
            FirstVariant,
            SecondVariant,
        }

        #[derive(Debug)]
        struct AnotherTest {
            uint8 pos0;
        }

        #[derive(Debug)]
        struct Test {
            uint8 pos0;
            AnotherTest pos1;
        }

        function createFooU16(uint16 x, uint16 y) external view returns (Foo);
        function createFoo2U16(uint16 x, uint16 y) external view returns (Foo,Foo);
        function createBazU16(uint16 x, uint16 y) external view returns (Baz);
        function createBaz2U16(uint16 x, uint16 y) external view returns (Baz,Baz);
        function multiValues1() external view returns (uint32[], uint128[], bool, uint64);
        function multiValues2() external view returns (uint8, bool, uint64);
        function echoVariant(TestEnum v) external view returns (TestEnum);
        function testValues(Test test) external view returns (uint8, uint8);
        function echo(uint128 x) external view returns (uint128);
        function getCopiedLocal() external view returns (uint128);
        function getConstant() external view returns (uint128);
        function getConstantLocal() external view returns (uint128);
        function getLocal(uint128 x) external view returns (uint128);
        function echoSignerWithInt(uint8 y) public view returns (uint8, address);
        function txContextProperties() public view returns (address, uint256, uint64, uint256, uint64, uint64, uint64, uint256);
        function fibonacci(uint64 n) external view returns (uint64);
        function sumSpecial(uint64 n) external view returns (uint64);
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = Example::new(address, provider.clone());

    let num = example.echo(123).call().await?;
    println!("echo(123) = {}", num);

    let num = example.getConstant().call().await?;
    println!("getConstant = {}", num);

    let num = example.getConstantLocal().call().await?;
    println!("getConstantLocal = {}", num);

    let num = example.getCopiedLocal().call().await?;
    println!("getCopiedLocal = {}", num);

    let num = example.getLocal(456).call().await?;
    println!("getLocal = {}", num);

    let tx_context = example.txContextProperties().call().await?;
    println!("txContextProperties:");
    println!("  - msg.sender: {:?}", tx_context._0);
    println!("  - msg.value: {}", tx_context._1);
    println!("  - block.number: {}", tx_context._2);
    println!("  - block.basefee: {}", tx_context._3);
    println!("  - block.gas_limit: {}", tx_context._4);
    println!("  - block.timestamp: {}", tx_context._5);
    println!("  - chainid: {}", tx_context._6);
    println!("  - tx.gas_price: {}", tx_context._7);

    let fib10 = example.fibonacci(10).call().await?;
    println!("fibonacci(10) = {}", fib10);

    let fib20 = example.fibonacci(20).call().await?;
    println!("fibonacci(20) = {}", fib20);

    let sum_special_2 = example.sumSpecial(2).call().await?;
    println!("sumSpecial(2) = {}", sum_special_2);

    let sum_special_4 = example.sumSpecial(4).call().await?;
    println!("sumSpecial(4) = {}", sum_special_4);

    let create_foo = example.createFooU16(55, 66).call().await?;
    println!("createFooU16(55, 66) = {:#?}", create_foo);

    let create_baz = example.createBazU16(55, 66).call().await?;
    println!("createBazU16(55, 66) = {:#?}", create_baz);

    let create_foo = example.createFoo2U16(55, 66).call().await?;
    println!(
        "createFoo2U16(55, 66) = {:#?} {:#?}",
        create_foo._0, create_foo._1
    );

    let create_baz = example.createBaz2U16(55, 66).call().await?;
    println!(
        "createBaz2U16(55, 66) = {:#?} {:#?}",
        create_baz._0, create_baz._1
    );

    let multi_values = example.multiValues1().call().await?;
    println!(
        "multiValues1 = ({:?}, {:?}, {}, {})",
        multi_values._0, multi_values._1, multi_values._2, multi_values._3
    );

    let multi_values = example.multiValues2().call().await?;
    println!(
        "multiValues2 = ({}, {}, {})",
        multi_values._0, multi_values._1, multi_values._2
    );

    let num = example.echo(123).call().await;
    println!("Example echo = {:?}", num);
    let echo_variant = example
        .echoVariant(Example::TestEnum::FirstVariant)
        .call()
        .await?;
    println!("echoVariant(FirstVariant) = {:?}", echo_variant);

    let echo_variant = example
        .echoVariant(Example::TestEnum::SecondVariant)
        .call()
        .await?;
    println!("echoVariant(SecondVariant) = {:?}", echo_variant);

    let test_values = example
        .testValues(Example::Test {
            pos0: 55,
            pos1: Example::AnotherTest { pos0: 66 },
        })
        .call()
        .await?;
    println!("testValues = ({}, {})", test_values._0, test_values._1);

    Ok(())
}
