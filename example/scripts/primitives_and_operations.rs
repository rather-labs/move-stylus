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
        function and(bool x, bool y) external returns (bool);
        function andU64(uint64 x, uint64 y) external returns (uint64);
        function castU8(uint128 x) external returns (uint8);
        function divU32(uint32 x, uint32 y) external returns (uint32);
        function greaterThanEqU32(uint32 a, uint32 b) external returns (bool);
        function greaterThanU64(uint64 a, uint64 b) external returns (bool);
        function lessThanEqU128(uint128 a, uint128 b) external returns (bool);
        function lessThanU256(uint256 a, uint256 b) external returns (bool);
        function modU16(uint16 x, uint16 y) external returns (uint16);
        function mulU64(uint64 x, uint64 y) external returns (uint64);
        function not(bool x) external returns (bool);
        function notTrue() external returns (bool);
        function or(bool x, bool y) external returns (bool);
        function orU256(uint256 x, uint256 y) external returns (uint256);
        function shiftLeftU32(uint32 x, uint8 slots) external returns (uint32);
        function shiftRightU16(uint16 x, uint8 slots) external returns (uint16);
        function subU128(uint128 x, uint128 y) external returns (uint128);
        function sumU256(uint256 x, uint256 y) external returns (uint256);
        function vecFromU256(uint256 x, uint256 y) external returns (uint256[]);
        function vecLenU128(uint128[] x) external returns (uint64);
        function vecPopBackU64(uint64[] x) external returns (uint64[]);
        function vecPushBackU16(uint16[] x, uint16 y) external returns (uint16[]);
        function vecSwapU32(uint32[] x, uint64 id1, uint64 id2) external returns (uint32[]);
        function xorU128(uint128 x, uint128 y) external returns (uint128);
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

    println!("==============================================================================");
    println!(" Testing primitive arithmetic operations");
    println!("==============================================================================");

    let res = example.castU8(42u128).call().await?;
    println!("castU8(42): {res}");
    assert_eq!(res, 42u8, "castU8(42) should return 42");

    let res = example
        .sumU256(U256::from(u128::MAX), U256::from(u128::MAX))
        .call()
        .await?;
    let expected_sum = U256::from(u128::MAX) + U256::from(u128::MAX);
    println!("sumU256(u128::MAX, u128::MAX): {res}");
    assert_eq!(
        res, expected_sum,
        "sumU256 should correctly add two u128::MAX values"
    );

    let res = example.subU128(u128::MAX, u128::MAX - 1).call().await?;
    println!("subU128(u128::MAX, u128::MAX - 1): {res}");
    assert_eq!(res, 1u128, "subU128 should return 1");

    let res = example.mulU64(u32::MAX as u64, 2).call().await?;
    println!("mulU64(u32::MAX, 2): {res}");
    assert_eq!(
        res,
        (u32::MAX as u64) * 2,
        "mulU64 should correctly multiply"
    );

    let res = example.divU32(u32::MAX, 2).call().await?;
    println!("divU32(u32::MAX, 2): {res}");
    assert_eq!(res, u32::MAX / 2, "divU32 should correctly divide");

    let res = example.modU16(100, 3).call().await?;
    println!("modU16(100, 3): {res}");
    assert_eq!(res, 100u16 % 3u16, "modU16 should return correct remainder");

    println!("\n==============================================================================");
    println!(" Testing bitwise operations");
    println!("==============================================================================");

    let res = example
        .orU256(
            U256::from(0xF0F0F0F0F0F0F0F0u128),
            U256::from(0x0F0F0F0F0F0F0F0Fu128),
        )
        .call()
        .await?;
    let expected_or = U256::from(0xF0F0F0F0F0F0F0F0u128) | U256::from(0x0F0F0F0F0F0F0F0Fu128);
    println!("orU256(0xF0F0F0F0F0F0F0F0, 0x0F0F0F0F0F0F0F0F): 0x{res:x}");
    assert_eq!(
        res, expected_or,
        "orU256 should perform bitwise OR correctly"
    );

    let res = example.xorU128(u128::MAX, u64::MAX as u128).call().await?;
    let expected_xor = u128::MAX ^ (u64::MAX as u128);
    println!("xorU128(u128::MAX, u64::MAX): 0x{res:x}");
    assert_eq!(
        res, expected_xor,
        "xorU128 should perform bitwise XOR correctly"
    );

    let res = example
        .andU64(u64::MAX, 0xF000FFFFFFFF000Fu64)
        .call()
        .await?;
    let expected_and = u64::MAX & 0xF000FFFFFFFF000Fu64;
    println!("andU64(u64::MAX, 0xF000FFFFFFFF000F): 0x{res:x}");
    assert_eq!(
        res, expected_and,
        "andU64 should perform bitwise AND correctly"
    );

    let res = example.shiftLeftU32(1, 31).call().await?;
    println!("shiftLeftU32(1, 31): 0x{res:x} ({res})");
    assert_eq!(res, 1u32 << 31, "shiftLeftU32 should shift left correctly");

    let res = example.shiftRightU16(0xFFFF, 15).call().await?;
    println!("shiftRightU16(0xFFFF, 15): 0x{res:x} ({res})");
    assert_eq!(
        res,
        0xFFFFu16 >> 15,
        "shiftRightU16 should shift right correctly"
    );

    println!("\n==============================================================================");
    println!(" Testing boolean operations");
    println!("==============================================================================");

    let res = example.notTrue().call().await?;
    println!("notTrue(): {res}");
    assert!(!res, "notTrue() should return false");

    let res = example.not(true).call().await?;
    println!("not(true): {res}");
    assert!(!res, "not(true) should return false");

    let res = example.not(false).call().await?;
    println!("not(false): {res}");
    assert!(res, "not(false) should return true");

    let res = example.and(true, false).call().await?;
    println!("and(true, false): {res}");
    assert!(!res, "and(true, false) should return false");

    let res = example.or(true, false).call().await?;
    println!("or(true, false): {res}");
    assert!(res, "or(true, false) should return true");

    println!("\n==============================================================================");
    println!(" Testing comparison operations");
    println!("==============================================================================");

    let res = example
        .lessThanU256(U256::from(10), U256::from(20))
        .call()
        .await?;
    println!("lessThanU256(10, 20): {res}");
    assert!(res, "10 should be less than 20");

    let res = example
        .lessThanU256(U256::from(20), U256::from(10))
        .call()
        .await?;
    println!("lessThanU256(20, 10): {res}");
    assert!(!res, "20 should not be less than 10");

    let res = example.lessThanEqU128(u128::MAX, u128::MAX).call().await?;
    println!("lessThanEqU128(u128::MAX, u128::MAX): {res}");
    assert!(res, "u128::MAX should be less than or equal to itself");

    let res = example
        .lessThanEqU128(u128::MAX - 1, u128::MAX)
        .call()
        .await?;
    println!("lessThanEqU128(u128::MAX - 1, u128::MAX): {res}");
    assert!(
        res,
        "u128::MAX - 1 should be less than or equal to u128::MAX"
    );

    let res = example
        .lessThanEqU128(u128::MAX, u128::MAX - 1)
        .call()
        .await?;
    println!("lessThanEqU128(u128::MAX, u128::MAX - 1): {res}");
    assert!(
        !res,
        "u128::MAX should not be less than or equal to u128::MAX - 1"
    );

    let res = example.greaterThanU64(100, 50).call().await?;
    println!("greaterThanU64(100, 50): {res}");
    assert!(res, "100 should be greater than 50");

    let res = example.greaterThanU64(50, 100).call().await?;
    println!("greaterThanU64(50, 100): {res}");
    assert!(!res, "50 should not be greater than 100");

    let res = example.greaterThanEqU32(200, 200).call().await?;
    println!("greaterThanEqU32(200, 200): {res}");
    assert!(res, "200 should be greater than or equal to itself");

    let res = example.greaterThanEqU32(200 - 1, 200).call().await?;
    println!("greaterThanEqU32(199, 200): {res}");
    assert!(!res, "199 should not be greater than or equal to 200");

    let res = example.greaterThanEqU32(200, 200 - 1).call().await?;
    println!("greaterThanEqU32(200, 199): {res}");
    assert!(res, "200 should be greater than or equal to 199");

    println!("\n==============================================================================");
    println!(" Testing vector operations");
    println!("==============================================================================");

    let res = example
        .vecFromU256(U256::from(1), U256::from(2))
        .call()
        .await?;
    println!("vecFromU256(1, 2): {res:?}");
    assert_eq!(
        res.len(),
        3,
        "vecFromU256 should create a vector with 3 elements"
    );
    assert_eq!(res[0], U256::from(1), "First element should be 1");
    assert_eq!(res[1], U256::from(2), "Second element should be 2");
    assert_eq!(res[2], U256::from(1), "Third element should be 1");

    let res = example.vecLenU128(vec![1, 2, 3, 4]).call().await?;
    println!("vecLenU128([1, 2, 3, 4]): {res}");
    assert_eq!(res, 4, "vecLenU128 should return length 4");

    let res = example.vecPopBackU64(vec![1, 2, 3, 4]).call().await?;
    println!("vecPopBackU64([1, 2, 3, 4]): {res:?}");
    assert_eq!(res.len(), 3, "vecPopBackU64 should remove last element");
    assert_eq!(
        res,
        vec![1u64, 2, 3],
        "vecPopBackU64 should return [1, 2, 3]"
    );

    let res = example.vecSwapU32(vec![1, 2, 3, 4], 0, 3).call().await?;
    println!("vecSwapU32([1, 2, 3, 4], 0, 3): {res:?}");
    assert_eq!(res.len(), 4, "vecSwapU32 should maintain length");
    assert_eq!(res[0], 4u32, "First element should be swapped to 4");
    assert_eq!(res[3], 1u32, "Last element should be swapped to 1");

    let res = example.vecPushBackU16(vec![1, 2, 3], 4).call().await?;
    println!("vecPushBackU16([1, 2, 3], 4): {res:?}");
    assert_eq!(res.len(), 4, "vecPushBackU16 should add one element");
    assert_eq!(
        res,
        vec![1u16, 2, 3, 4],
        "vecPushBackU16 should return [1, 2, 3, 4]"
    );

    Ok(())
}
