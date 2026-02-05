//! Example on how to test revert errors from a deployed `revert_errors` contract.
//! This example demonstrates how to catch and decode custom revert errors from Move contracts
//! compiled to WASM with Stylus.

use std::str::FromStr;
use std::sync::Arc;

use dotenv::dotenv;
use eyre::eyre;

use alloy::{
    primitives::Address,
    primitives::{U256, address},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
    sol_types::SolError,
    transports::http::reqwest::Url,
};

sol!(
    #[sol(rpc)]
    #[allow(missing_docs)]
    #[allow(clippy::too_many_arguments)]
    contract RevertErrors {
        // Error structs matching the Move contract
        #[derive(Debug, PartialEq)]
        error BasicError(string);

        #[derive(Debug, PartialEq)]
        error CustomError(
            string error_message,
            uint64 error_code,
        );

        #[derive(Debug, PartialEq)]
        error CustomError2(
            bool a,
            uint8 b,
            uint16 c,
            uint32 d,
            uint64 e,
            uint128 f,
            uint256 g,
            address h,
            uint8 i, // MyEnum: A=0, B=1, C=2
        );

        #[derive(Debug, PartialEq)]
        error CustomError3(
            uint32[] a,
            uint128[] b,
            uint64[][] c,
        );

        #[derive(Debug, PartialEq)]
        struct NestedStruct {
            string _0;
        }

        #[derive(Debug, PartialEq)]
        struct NestedStruct2 {
            string a;
            uint64 b;
        }

        #[derive(Debug, PartialEq)]
        error CustomError4(
            NestedStruct a,
            NestedStruct2 b,
        );

        // The standard EVM error type
        #[derive(Debug, PartialEq)]
        error Error(string);

        // Functions from the Move contract
        function revertStandardError(string s) external;
        function revertCustomError(string s, uint64 code) external;
        function revertCustomError2(bool a, uint8 b, uint16 c, uint32 d, uint64 e, uint128 f, uint256 g, address h, uint8 i) external;
        function revertCustomError3(uint32[] a, uint128[] b, uint64[][] c) external;
        function revertCustomError4(string a, uint64 b) external;
        function abortWithCleverError() external;
        function abortWithAnotherCleverError() external;
    }
);

/// Helper function to check and decode revert errors from a transaction result
async fn check_and_decode_revert_error<E: SolError + PartialEq + std::fmt::Debug>(
    expected_error: E,
    pending_tx_result: Result<
        alloy::providers::PendingTransactionBuilder<alloy::network::Ethereum>,
        alloy::contract::Error,
    >,
) where
    E::Parameters<'static>: alloy::sol_types::SolType,
{
    match pending_tx_result {
        Ok(_pending_tx) => {
            panic!("Transaction should have failed with a revert error");
        }
        Err(e) => {
            match e.as_decoded_error::<E>() {
                Some(decoded_error) => {
                    println!("  ✓ Error decoded successfully");
                    println!("  Decoded error: {decoded_error:?}");
                    assert_eq!(
                        decoded_error, expected_error,
                        "Decoded error should match expected error"
                    );
                    println!("  ✓ Error matches expected value");
                }
                None => {
                    println!("  ✗ Could not decode revert data as {}", E::SIGNATURE);
                    let revert_data = e.as_revert_data().unwrap();
                    println!("  Revert data: {revert_data:?}");
                    panic!("Failed to decode error as expected type");
                }
            };
        }
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;

    let contract_address = std::env::var("CONTRACT_ADDRESS_REVERT_ERRORS")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_REVERT_ERRORS"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let contract = RevertErrors::new(address, provider.clone());

    println!("==============================================================================");
    println!(" Testing revert errors from revert_errors.move contract");
    println!("==============================================================================");

    println!("\nTest 1: BasicError (string message)");
    println!("  Calling revertStandardError(\"Not enough Ether provided.\")");
    let error_ = RevertErrors::BasicError::new((String::from("Not enough Ether provided."),));
    let pending_tx = contract
        .revertStandardError(String::from("Not enough Ether provided."))
        .send()
        .await;
    check_and_decode_revert_error(error_, pending_tx).await;

    println!("\nTest 2: CustomError (string message + error code)");
    println!("  Calling revertCustomError(\"Not enough Ether provided.\", 42)");
    let custom_error =
        RevertErrors::CustomError::new((String::from("Not enough Ether provided."), 42));
    let pending_tx = contract
        .revertCustomError(String::from("Not enough Ether provided."), 42)
        .send()
        .await;
    check_and_decode_revert_error(custom_error, pending_tx).await;

    println!("\nTest 3: CustomError2 (multiple primitive types)");
    println!("  Calling revertCustomError2 with various types");
    let custom_error2 = RevertErrors::CustomError2::new((
        true,
        2,
        3,
        4,
        5,
        5,
        U256::from(5),
        address!("0xffffffffffffffffffffffffffffffffffffffff"),
        0,
    ));
    let pending_tx = contract
        .revertCustomError2(
            true,
            2,
            3,
            4,
            5,
            5,
            U256::from(5),
            address!("0xffffffffffffffffffffffffffffffffffffffff"),
            0,
        )
        .send()
        .await;
    check_and_decode_revert_error(custom_error2, pending_tx).await;

    println!("\nTest 4: CustomError3 (arrays and nested arrays)");
    println!("  Calling revertCustomError3 with arrays");
    let custom_error3 = RevertErrors::CustomError3::new((
        vec![1, 2, 3],
        vec![4, 5],
        vec![vec![6, 7, 8], vec![9, 10, 11]],
    ));
    let pending_tx = contract
        .revertCustomError3(
            vec![1, 2, 3],
            vec![4, 5],
            vec![vec![6, 7, 8], vec![9, 10, 11]],
        )
        .send()
        .await;
    check_and_decode_revert_error(custom_error3, pending_tx).await;

    println!("\nTest 5: CustomError4 (nested structs)");
    println!("  Calling revertCustomError4 with nested structs");
    let custom_error4 = RevertErrors::CustomError4::new((
        RevertErrors::NestedStruct {
            _0: String::from("Custom error message"),
        },
        RevertErrors::NestedStruct2 {
            a: String::from("Custom error message"),
            b: 42,
        },
    ));
    let pending_tx = contract
        .revertCustomError4(String::from("Custom error message"), 42)
        .send()
        .await;
    check_and_decode_revert_error(custom_error4, pending_tx).await;

    println!("\nTest 6: abortWithCleverError");
    println!("  Calling abortWithCleverError");
    let pending_tx = contract.abortWithCleverError().send().await;
    check_and_decode_revert_error(
        RevertErrors::Error::new((String::from("Error for testing #[error] macro"),)),
        pending_tx,
    )
    .await;

    println!("\nTest 7: abortWithAnotherCleverError");
    println!("  Calling abortWithAnotherCleverError");
    let pending_tx = contract.abortWithAnotherCleverError().send().await;
    check_and_decode_revert_error(
        RevertErrors::Error::new((String::from("Another error for testing clever errors"),)),
        pending_tx,
    )
    .await;

    println!("\n==============================================================================");
    println!(" ✓ All revert error tests completed successfully!");
    println!("==============================================================================");
    Ok(())
}
