//! on how to interact with a deployed `stylus-hello-world` contract using defaults.
//! This example uses ethers-rs to instantiate the contract using a Solidity ABI.
//! Then, it attempts to check the current counter value, increment it via a tx,
//! and check the value again. The deployed contract is fully written in Rust and compiled to WASM
//! but with Stylus, it is accessible just as a normal Solidity smart contract is via an ABI.

use dotenv::dotenv;
use ethers::{
    contract::abigen, middleware::SignerMiddleware, providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};

use eyre::eyre;
use std::str::FromStr;
use std::sync::Arc;





#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_PRIMITIVES")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_PRIMITIVES"))?;

    abigen!(
        Example,
        r#"
        [
            function castU8(uint128 x) external view returns (uint8)
            function sumU256(uint256 x, uint256 y) external view returns (uint256)
            function subU128(uint128 x, uint128 y) external view returns (uint128)
            function mulU64(uint64 x, uint64 y) external view returns (uint64)
            function divU32(uint32 x, uint32 y) external view returns (uint32)
            function modU16(uint16 x, uint16 y) external view returns (uint16)
            function orU256(uint256 x, uint256 y) external view returns (uint256)
            function xorU128(uint128 x, uint128 y) external view returns (uint128)
            function andU64(uint64 x, uint64 y) external view returns (uint64)
            function shiftLeftU32(uint32 x, uint8 y) external view returns (uint32)
            function shiftRightU16(uint16 x, uint8 y) external view returns (uint16)
            function notTrue() external view returns (bool)
            function not(bool x) external view returns (bool)
            function and(bool x, bool y) external view returns (bool)
            function or(bool x, bool y) external view returns (bool)
            function lessThanU256(uint256 x, uint256 y) external view returns (bool)
            function lessThanEqU128(uint128 x, uint128 y) external view returns (bool)
            function greaterThanU64(uint64 x, uint64 y) external view returns (bool)
            function greaterThanEqU32(uint32 x, uint32 y) external view returns (bool)
            function vecFromU256(uint256 x, uint256 y) external view returns (uint256[])
            function vecLenU128(uint128[] x) external view returns (uint64)
            function vecPopBackU64(uint64[] x) external view returns (uint64[])
            function vecSwapU32(uint32[] x, uint64 i, uint64 j) external view returns (uint32[])
            function vecPushBackU16(uint16[] x, uint16 y) external view returns (uint16[])
        ]
        "#
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

    println!("Primitive arithmetic operations");

    let res = example.cast_u8(42u128).call().await?;
    println!("castU8: {}", res);

    let res = example.sum_u256(U256::from(u128::MAX), U256::from(u128::MAX)).call().await?;
    println!("sumU256: {}", res);

    let res = example.sub_u128(u128::MAX, u128::MAX - 1).call().await?;
    println!("subU128: {}", res);

    let res = example.mul_u64(u32::MAX as u64, 2).call().await?;
    println!("mulU64: {}", res);

    let res = example.div_u32(u32::MAX, 2).call().await?;
    println!("divU32: {}", res);

    let res = example.mod_u16(100, 3).call().await?;
    println!("modU16: {}", res);

    println!("\nBitwise operations");

    let res = example.or_u256(U256::from(0xF0F0F0F0F0F0F0F0u128), U256::from(0x0F0F0F0F0F0F0F0Fu128)).call().await?;
    println!("orU256: 0x{:x}", res);

    let res = example.xor_u128(u128::MAX, u64::MAX as u128).call().await?;
    println!("xorU128: 0x{:x}", res);

    let res = example.and_u64(u64::MAX, 0xF000FFFFFFFF000Fu64).call().await?;
    println!("andU64: 0x{:x}", res);

    let res = example.shift_left_u32(1, 31).call().await?;
    println!("shiftLeftU32: 0x{:x}", res);

    let res = example.shift_right_u16(0xFFFF, 15).call().await?;
    println!("shiftRightU16: 0x{:x}", res);

    println!("\nBoolean operations");

    let res = example.not_true().call().await?;
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

    let res = example.less_than_u256(U256::from(10), U256::from(20)).call().await?;
    println!("lessThanU256(10, 20): {}", res);

    let res = example.less_than_u256(U256::from(20), U256::from(10)).call().await?;
    println!("lessThanU256(20, 10): {}", res);

    let res = example.less_than_eq_u128(u128::MAX, u128::MAX).call().await?;
    println!("lessThanEqU128(u128::MAX, u128::MAX): {}", res);

    let res = example.less_than_eq_u128(u128::MAX - 1, u128::MAX).call().await?;
    println!("lessThanEqU128(u128::MAX - 1, u128::MAX): {}", res);

    let res = example.less_than_eq_u128(u128::MAX, u128::MAX - 1).call().await?;
    println!("lessThanEqU128(u128::MAX, u128::MAX - 1): {}", res);

    let res = example.greater_than_u64(100, 50).call().await?;
    println!("greaterThanU64(100, 50): {}", res);

    let res = example.greater_than_u64(50, 100).call().await?;
    println!("greaterThanU64(50, 100): {}", res);

    let res = example.greater_than_eq_u32(200, 200).call().await?;
    println!("greaterThanEqU32(200, 200): {}", res);

    let res = example.greater_than_eq_u32(200 - 1, 200).call().await?;
    println!("greaterThanEqU32(200 - 1, 200): {}", res);

    let res = example.greater_than_eq_u32(200, 200 - 1).call().await?;
    println!("greaterThanEqU32(200, 200 - 1): {}", res);

    println!("\nVector operations");

    let res = example.vec_from_u256(U256::from(1), U256::from(2)).call().await?;
    println!("vecFromU256(1, 2): {:?}", res);

    let res = example.vec_len_u128(vec![1, 2, 3, 4]).call().await?;
    println!("vecLenU128([1, 2, 3, 4]): {}", res);

    let res = example.vec_pop_back_u64(vec![1, 2, 3, 4]).call().await?;
    println!("vecPopBackU64([1, 2, 3, 4]): {:?}", res);

    let res = example.vec_swap_u32(vec![1, 2, 3, 4], 0, 3).call().await?;
    println!("vecSwapU32([1, 2, 3, 4], 0, 3): {:?}", res);

    let res = example.vec_push_back_u16(vec![1, 2, 3], 4).call().await?;
    println!("vecPushBackU16([1, 2, 3], 4): {:?}", res);

    Ok(())
}

