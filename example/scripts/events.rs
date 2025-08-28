use std::{str::FromStr, sync::Arc};

use alloy::{
    primitives::Address, providers::ProviderBuilder, signers::local::PrivateKeySigner,
    transports::http::reqwest::Url,
};
use alloy_sol_types::{SolValue, sol};
use dotenv::dotenv;
use eyre::eyre;

sol!(
    #[sol(rpc)]
    #[allow(missing_docs)]
    contract Example {
        #[derive(Debug, PartialEq)]
        struct TestEvent1 {
            uint32 n;
        }

        #[derive(Debug, PartialEq)]
        struct TestEvent2 {
            uint32 a;
            uint8[] b;
            uint128 c;
        }

        #[derive(Debug, PartialEq)]
        struct TestEvent3 {
            TestEvent1 a;
            TestEvent2 b;
        }

        #[derive(Debug, PartialEq)]
        struct TestGenericEvent1 {
            uint32 o;
            bool p;
            TestEvent1 q;
        }
        function emitTestEvent1(uint32 n) public view;
        function emitTestEvent2(uint32 a, uint8[] b, uint128 c) public view;
        function emitTestEvent3(TestEvent1 a, TestEvent2 b) public view;
        function emitTestEventGeneric1(uint32 o, bool p, TestEvent1 q) public view;
    }
);

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenv().ok();
    let priv_key = std::env::var("PRIV_KEY").map_err(|_| eyre!("No {} env var set", "PRIV_KEY"))?;
    let rpc_url = std::env::var("RPC_URL").map_err(|_| eyre!("No {} env var set", "RPC_URL"))?;
    let contract_address = std::env::var("CONTRACT_ADDRESS_EVENTS")
        .map_err(|_| eyre!("No {} env var set", "CONTRACT_ADDRESS_EVENTS"))?;

    let signer = PrivateKeySigner::from_str(&priv_key)?;

    let provider = Arc::new(
        ProviderBuilder::new()
            .wallet(signer)
            .with_chain_id(412346)
            .connect_http(Url::from_str(&rpc_url).unwrap()),
    );
    let address = Address::from_str(&contract_address)?;
    let example = Example::new(address, provider.clone());

    // Emit test event 1
    let pending_tx = example.emitTestEvent1(43).send().await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent1 { n: 43 };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let data = log.data().data.0.clone();
        let decoded_event = <Example::TestEvent1 as SolValue>::abi_decode(&data)?;
        assert_eq!(event, decoded_event);
        println!("Decoded event data = {:?}", decoded_event);
    }

    // Emit test event 2
    let pending_tx = example
        .emitTestEvent2(43, vec![1, 2, 3], 1234)
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent2 {
        a: 43,
        b: vec![1, 2, 3],
        c: 1234,
    };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let data = log.data().data.0.clone();
        let decoded_event = <Example::TestEvent2 as SolValue>::abi_decode(&data)?;
        println!("Decoded event data = {:?}", decoded_event);
        assert_eq!(event, decoded_event);
    }

    // Emit test event 3
    let pending_tx = example
        .emitTestEvent3(
            Example::TestEvent1 { n: 43 },
            Example::TestEvent2 {
                a: 43,
                b: vec![1, 2, 3],
                c: 1234,
            },
        )
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestEvent3 {
        a: Example::TestEvent1 { n: 43 },
        b: Example::TestEvent2 {
            a: 43,
            b: vec![1, 2, 3],
            c: 1234,
        },
    };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let data = log.data().data.0.clone();
        let decoded_event = <Example::TestEvent3 as SolValue>::abi_decode(&data)?;
        println!("Decoded event data = {:?}", decoded_event);
        assert_eq!(event, decoded_event);
    }

    // Emit test event with generics 1
    let pending_tx = example
        .emitTestEventGeneric1(43, true, Example::TestEvent1 { n: 43 })
        .send()
        .await?;
    let receipt = pending_tx.get_receipt().await?;
    let event = Example::TestGenericEvent1 {
        o: 43,
        p: true,
        q: Example::TestEvent1 { n: 43 },
    };

    // Decode the event data
    let logs = receipt.logs();
    for log in logs {
        let data = log.data().data.0.clone();
        let decoded_event = <Example::TestGenericEvent1 as SolValue>::abi_decode(&data)?;
        println!("Decoded event data = {:?}", decoded_event);
        assert_eq!(event, decoded_event);
    };

    Ok(())
}
