//! on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use alloy::{
    primitives::{Address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
    transports::http::reqwest::Url,
};
use dotenv::dotenv;
use eyre::eyre;
use std::{str::FromStr, sync::Arc};

sol! {
    #[sol(rpc)]
    #[allow(missing_docs)]
    contract PrimitiveOperations {
        function castU8(uint128 x) external view returns (uint8);
        function sumU256(uint256 x, uint256 y) external view returns (uint256);
        function subU128(uint128 x, uint128 y) external view returns (uint128);
        function mulU64(uint64 x, uint64 y) external view returns (uint64);
        function divU32(uint32 x, uint32 y) external view returns (uint32);
        function modU16(uint16 x, uint16 y) external view returns (uint16);

        function orU256(uint256 x, uint256 y) external view returns (uint256);
        function xorU128(uint128 x, uint128 y) external view returns (uint128);
        function andU64(uint64 x, uint64 y) external view returns (uint64);
        function shiftLeftU32(uint32 x, uint8 y) external view returns (uint32);
        function shiftRightU16(uint16 x, uint8 y) external view returns (uint16);

        function notTrue() external pure returns (bool);
        function not(bool x) external pure returns (bool);
        function and(bool x, bool y) external pure returns (bool);
        function or(bool x, bool y) external pure returns (bool);

        function lessThanU256(uint256 x, uint256 y) external view returns (bool);
        function lessThanEqU128(uint128 x, uint128 y) external view returns (bool);
        function greaterThanU64(uint64 x, uint64 y) external view returns (bool);
        function greaterThanEqU32(uint32 x, uint32 y) external view returns (bool);

        function vecFromU256(uint256 a, uint256 b) external pure returns (uint256[]);
        function vecLenU128(uint128[] memory arr) external pure returns (uint256);
        function vecPopBackU64(uint64[] memory arr) external pure returns (uint64[] memory);
        function vecSwapU32(uint32[] memory arr, uint64 i, uint64 j)
            external
            pure
            returns (uint32[] memory);
        function vecPushBackU16(uint16[] memory arr, uint16 value)
            external
            pure
            returns (uint16[] memory);
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_PRIMITIVES")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_PRIMITIVES"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = PrimitiveOperations::new(address, provider.clone());

    println!("Primitive arithmetic operations");

    let res = example.castU8(42u128).call().await?;
    println!("castU8: {}", res);

    let res = example
        .sumU256(U256::from(u128::MAX), U256::from(u128::MAX))
        .call()
        .await?;
    println!("sumU256: {}", res);

    let res = example.subU128(u128::MAX, u128::MAX - 1).call().await?;
    println!("subU128: {}", res);

    let res = example.mulU64(u32::MAX as u64, 2).call().await?;
    println!("mulU64: {}", res);

    let res = example.divU32(u32::MAX, 2).call().await?;
    println!("divU32: {}", res);

    let res = example.modU16(100, 3).call().await?;
    println!("modU16: {}", res);

    println!("\nBitwise operations");

    let res = example
        .orU256(
            U256::from(0xF0F0F0F0F0F0F0F0u128),
            U256::from(0x0F0F0F0F0F0F0F0Fu128),
        )
        .call()
        .await?;
    println!("orU256: 0x{:x}", res);

    let res = example.xorU128(u128::MAX, u64::MAX as u128).call().await?;
    println!("xorU128: 0x{:x}", res);

    let res = example
        .andU64(u64::MAX, 0xF000FFFFFFFF000Fu64)
        .call()
        .await?;
    println!("andU64: 0x{:x}", res);

    let res = example.shiftLeftU32(1, 31).call().await?;
    println!("shiftLeftU32: 0x{:x}", res);

    let res = example.shiftRightU16(0xFFFF, 15).call().await?;
    println!("shiftRightU16: 0x{:x}", res);

    println!("\nBoolean operations");

    let res = example.notTrue().call().await?;
    println!("notTrue: {}", res);

    let res = example.not(true).call().await?;
    println!("not(true): {}", res);

    let res = example.not(false).call().await?;
    println!("not(false): {}", res);

    let res = example.and(true, false).call().await?;
    println!("and(true, false): {}", res);

    let res = example.or(true, false).call().await?;
    println!("or(true, false): {}", res);

    println!("\nComparison operations");

    let res = example
        .lessThanU256(U256::from(10), U256::from(20))
        .call()
        .await?;
    println!("lessThanU256(10, 20): {}", res);

    let res = example
        .lessThanU256(U256::from(20), U256::from(10))
        .call()
        .await?;
    println!("lessThanU256(20, 10): {}", res);

    let res = example.lessThanEqU128(u128::MAX, u128::MAX).call().await?;
    println!("lessThanEqU128(u128::MAX, u128::MAX): {}", res);

    let res = example
        .lessThanEqU128(u128::MAX - 1, u128::MAX)
        .call()
        .await?;
    println!("lessThanEqU128(u128::MAX - 1, u128::MAX): {}", res);

    let res = example
        .lessThanEqU128(u128::MAX, u128::MAX - 1)
        .call()
        .await?;
    println!("lessThanEqU128(u128::MAX, u128::MAX - 1): {}", res);

    let res = example.greaterThanU64(100, 50).call().await?;
    println!("greaterThanU64(100, 50): {}", res);

    let res = example.greaterThanU64(50, 100).call().await?;
    println!("greaterThanU64(50, 100): {}", res);

    let res = example.greaterThanEqU32(200, 200).call().await?;
    println!("greaterThanEqU32(200, 200): {}", res);

    let res = example.greaterThanEqU32(200 - 1, 200).call().await?;
    println!("greaterThanEqU32(200 - 1, 200): {}", res);

    let res = example.greaterThanEqU32(200, 200 - 1).call().await?;
    println!("greaterThanEqU32(200, 200 - 1): {}", res);

    println!("\nVector operations");

    let res = example
        .vecFromU256(U256::from(1), U256::from(2))
        .call()
        .await?;
    println!("vecFromU256(1, 2): {:?}", res);

    let res = example.vecLenU128(vec![1, 2, 3, 4]).call().await?;
    println!("vecLenU128([1, 2, 3, 4]): {}", res);

    let res = example.vecPopBackU64(vec![1, 2, 3, 4]).call().await?;
    println!("vecPopBackU64([1, 2, 3, 4]): {:?}", res);

    let res = example.vecSwapU32(vec![1, 2, 3, 4], 0, 3).call().await?;
    println!("vecSwapU32([1, 2, 3, 4], 0, 3): {:?}", res);

    let res = example.vecPushBackU16(vec![1, 2, 3], 4).call().await?;
    println!("vecPushBackU16([1, 2, 3], 4): {:?}", res);

    Ok(())
}
