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

        #[derive(Debug, PartialEq)]
        event ErrorEvent(
            string s,
        );


        #[derive(Debug, PartialEq)]
        event OkEvent(
            uint32 indexed c,
        );

        #[derive(Debug, PartialEq)]
        event ReceiveEvent (
            address indexed sender,
            uint32 data_length,
        );

        // Functions from the Move contract
        function revertStandardError(string s) external;
        function revertCustomError(string s, uint64 code) external;
        function revertCustomError2(bool a, uint8 b, uint16 c, uint32 d, uint64 e, uint128 f, uint256 g, address h, uint8 i) external;
        function revertCustomError3(uint32[] a, uint128[] b, uint64[][] c) external;
        function revertCustomError4(string a, uint64 b) external;
        function exampleWithRevert(uint32 a, uint32 b, string s) external;
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
        Ok(pending_tx) => {
            println!("Transaction sent successfully: {pending_tx:?}");
        }
        Err(e) => {
            match e.as_decoded_error::<E>() {
                Some(decoded_error) => {
                    println!("Decoded error: {decoded_error:?}");
                    assert_eq!(decoded_error, expected_error);
                }
                None => {
                    println!("Could not decode revert data as {}", E::SIGNATURE);
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

    println!("Testing revert errors from revert_errors.move contract\n");

    // Test 1
    println!("Test 1: Testing revert BasicError");
    let error_ = RevertErrors::BasicError::new((String::from("Not enough Ether provided."),));
    let pending_tx = contract
        .revertStandardError(String::from("Not enough Ether provided."))
        .send()
        .await;

    check_and_decode_revert_error(error_, pending_tx).await;

    // Test 2
    println!("Test 2: Testing revert CustomError");
    let custom_error =
        RevertErrors::CustomError::new((String::from("Not enough Ether provided."), 42));
    let pending_tx = contract
        .revertCustomError(String::from("Not enough Ether provided."), 42)
        .send()
        .await;
    check_and_decode_revert_error(custom_error, pending_tx).await;

    // Test 3
    println!("Test 3: Testing revert CustomError2");
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

    // Test 4
    println!("Test 4: Testing revert CustomError3");
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

    // Test 5
    println!("Test 5: Testing revert CustomError4");
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

    println!("\n✓ All revert error tests completed!");
    Ok(())
}
