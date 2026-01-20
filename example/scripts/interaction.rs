//! Example on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.
use alloy::primitives::{U256, address};
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
        event NewUID(bytes32 indexed uid);

        #[derive(Debug, PartialEq)]
        struct AnotherTest {
            uint8 pos0;
        }

        #[derive(Debug, PartialEq)]
        struct Bar {
            uint32 a;
            uint128 b;
        }

        #[derive(Debug, PartialEq)]
        struct BazUint16 {
            uint16 c;
            Bar d;
            address e;
            bool f;
            uint64 g;
            uint256 h;
        }

        #[derive(Debug, PartialEq)]
        struct FooUint16 {
            uint16 c;
            Bar d;
            address e;
            bool f;
            uint64 g;
            uint256 h;
            uint32[] i;
        }

        #[derive(Debug, PartialEq)]
        struct Test {
            uint8 pos0;
            AnotherTest pos1;
        }

        #[derive(Debug, PartialEq)]
        enum TestEnum {
            FirstVariant,
            SecondVariant,
        }

        function createBaz2U16(uint16 a, uint16 _b) external returns (BazUint16, BazUint16);
        function createBazU16(uint16 a, uint16 _b) external returns (BazUint16);
        function createFoo2U16(uint16 a, uint16 b) external returns (FooUint16, FooUint16);
        function createFooU16(uint16 a, uint16 b) external returns (FooUint16);
        function echo(uint128 x) external returns (uint128);
        function echo2(uint128 x, uint128 y) external returns (uint128);
        function echoVariant(TestEnum x) external returns (TestEnum);
        function fibonacci(uint64 n) external returns (uint64);
        function getConstant() external returns (uint128);
        function getConstantLocal() external returns (uint128);
        function getCopiedLocal() external returns (uint128);
        function getLocal(uint128 _z) external returns (uint128);
        function multiValues1() external returns (uint32[], uint128[], bool, uint64);
        function multiValues2() external returns (uint8, bool, uint64);
        function sumSpecial(uint64 n) external returns (uint64);
        function testValues(Test test) external returns (uint8, uint8);
        function txContextProperties() external returns (address, uint256, uint64, uint256, uint64, uint64, uint64, uint256);
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

    // ==================== Basic Echo & Locals ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║         Basic Echo & Locals          ║");
    println!("╚══════════════════════════════════════╝");

    let res = example.echo(123).call().await?;
    assert_eq!(res, 123u128);
    println!("  ✓ echo(123) = {res}");

    let res = example.getConstant().call().await?;
    assert_eq!(res, 128128128u128);
    println!("  ✓ getConstant() = {res}");

    let res = example.getConstantLocal().call().await?;
    assert_eq!(res, 128128128u128);
    println!("  ✓ getConstantLocal() = {res}");

    let res = example.getCopiedLocal().call().await?;
    assert_eq!(res, 100u128);
    println!("  ✓ getCopiedLocal() = {res}");

    let res = example.getLocal(456).call().await?;
    assert_eq!(res, 50u128);
    println!("  ✓ getLocal(456) = {res}");

    // ==================== Transaction Context ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║        Transaction Context           ║");
    println!("╚══════════════════════════════════════╝");

    let tx_context = example.txContextProperties().call().await?;
    println!("  msg.sender:      {:?}", tx_context._0);
    println!("  msg.value:       {}", tx_context._1);
    println!("  block.number:    {}", tx_context._2);
    println!("  block.basefee:   {}", tx_context._3);
    println!("  block.gas_limit: {}", tx_context._4);
    println!("  block.timestamp: {}", tx_context._5);
    println!("  chainid:         {}", tx_context._6);
    println!("  tx.gas_price:    {}", tx_context._7);

    // ==================== Fibonacci ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║             Fibonacci                ║");
    println!("╚══════════════════════════════════════╝");

    let res = example.fibonacci(10).call().await?;
    assert_eq!(res, 55u64);
    println!("  ✓ fibonacci(10) = {res}");

    let res = example.fibonacci(20).call().await?;
    assert_eq!(res, 6765u64);
    println!("  ✓ fibonacci(20) = {res}");

    // ==================== Sum Special ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║            Sum Special               ║");
    println!("╚══════════════════════════════════════╝");

    let res = example.sumSpecial(2).call().await?;
    assert_eq!(res, 7u64);
    println!("  ✓ sumSpecial(2) = {res}");

    let res = example.sumSpecial(4).call().await?;
    assert_eq!(res, 14u64);
    println!("  ✓ sumSpecial(4) = {res}");

    // ==================== Struct Creation ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║          Struct Creation             ║");
    println!("╚══════════════════════════════════════╝");

    let expected_bar = Example::Bar { a: 42, b: 4242 };
    let expected_addr = address!("0000000000000000000000000000000000007357");

    let expected_foo = Example::FooUint16 {
        c: 66,
        d: expected_bar.clone(),
        e: expected_addr,
        f: true,
        g: 1,
        h: U256::from(2),
        i: vec![0xFFFFFFFF],
    };

    let expected_baz = Example::BazUint16 {
        c: 55,
        d: expected_bar.clone(),
        e: expected_addr,
        f: true,
        g: 1,
        h: U256::from(2),
    };

    let res = example.createFooU16(55, 66).call().await?;
    assert_eq!(res, expected_foo);
    println!("  ✓ createFooU16(55, 66)");

    let res = example.createBazU16(55, 66).call().await?;
    assert_eq!(res, expected_baz);
    println!("  ✓ createBazU16(55, 66)");

    let res = example.createFoo2U16(55, 66).call().await?;
    assert_eq!(res._0, expected_foo);
    assert_eq!(res._1, expected_foo);
    println!("  ✓ createFoo2U16(55, 66)");

    let res = example.createBaz2U16(55, 66).call().await?;
    assert_eq!(res._0, expected_baz);
    assert_eq!(res._1, expected_baz);
    println!("  ✓ createBaz2U16(55, 66)");

    // ==================== Multiple Return Values ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║       Multiple Return Values         ║");
    println!("╚══════════════════════════════════════╝");

    let res = example.multiValues1().call().await?;
    assert_eq!(res._0, vec![0xFFFFFFFF, 0xFFFFFFFF]);
    assert_eq!(res._1, vec![0xFFFFFFFFFF_u128]);
    assert!(res._2);
    assert_eq!(res._3, 42u64);
    println!(
        "  ✓ multiValues1(): (uint32[], uint128[], bool, uint64) = ({:?}, {:?}, {}, {})",
        res._0, res._1, res._2, res._3
    );

    let res = example.multiValues2().call().await?;
    assert_eq!(res._0, 84u8);
    assert!(res._1);
    assert_eq!(res._2, 42u64);
    println!(
        "  ✓ multiValues2(): (u8, bool, uint64) = ({}, {}, {})",
        res._0, res._1, res._2
    );

    // ==================== Enum & Nested Struct ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║       Enum & Nested Struct           ║");
    println!("╚══════════════════════════════════════╝");

    let res = example
        .echoVariant(Example::TestEnum::FirstVariant)
        .call()
        .await?;
    assert_eq!(res, Example::TestEnum::FirstVariant);
    println!("  ✓ echoVariant(FirstVariant) = {res:?}");

    let res = example
        .echoVariant(Example::TestEnum::SecondVariant)
        .call()
        .await?;
    assert_eq!(res, Example::TestEnum::SecondVariant);
    println!("  ✓ echoVariant(SecondVariant) = {res:?}");

    let res = example
        .testValues(Example::Test {
            pos0: 55,
            pos1: Example::AnotherTest { pos0: 66 },
        })
        .call()
        .await?;
    assert_eq!(res._0, 55);
    assert_eq!(res._1, 66);
    println!(
        "  ✓ testValues({{pos0: 55, pos1: {{pos0: 66}}}}) = ({}, {})",
        res._0, res._1
    );

    // ==================== Done ====================
    println!("\n╔══════════════════════════════════════╗");
    println!("║        ✓ All tests passed!           ║");
    println!("╚══════════════════════════════════════╝\n");

    Ok(())
}
