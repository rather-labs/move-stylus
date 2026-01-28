# Deploy and Interact

In this section, we will guide you through the process of deploying your Move smart contract to the [Arbitrum's Nitro devnode]() using the `move-stylus` CLI and interacting with it using foundry's `cast` command.

## Prerequisites

- To run Arbitrum's Nitro devnode, check the [official documentation](https://docs.arbitrum.io/run-arbitrum-node/run-nitro-dev-node).
- To install foundry, follow the instructions in the [foundry book](https://book.getfoundry.sh/getting-started/installation).

## Deploying the Contract

To deploy the counter contract, make sure you are running the Arbitrum Nitro devnode. Open a terminal, navigate to the root directory of your project (`counter`).

First build the project:

```bash
move-stylus build
```

And then run the following command:

```bash
move-stylus deploy --contract-name counter --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659
```

> [!WARNING]
> The private key used in this example is for demonstration purposes only. Do not use it in production or with real funds.
>
> It corresponds to the address `0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E`, which is pre-funded in the Nitro devnode.

You should see output similar to:

```
Deploying contract 'counter' to endpoint 'http://localhost:8547' using provided private key...
stripped custom section from user wasm to remove any sensitive data
contract size: 1.9 KiB (1959 bytes)
wasm data fee: 0.000058 ETH (originally 0.000049 ETH with 20% bump)
deployed code at address: 0x525c2aba45f66987217323e8a05ea400c65d06dc
deployment tx hash: 0x641b8e7bf3207d61e011a1bc4a18c92d912f958d3be32911a50d8cd6296cff6b
contract activated and ready onchain with tx hash: 0xea84d26e12e89968c1f929263a27d92cdf8ea142e89a3f9c718ecb6a62444c6f
```

Take note of the deployed contract address (in this example, `0x525c2aba45f66987217323e8a05ea400c65d06dc`), as you will need it to interact with the contract.

To make things easier, you can save the contract address in an environment variable:

```bash
export CONTRACT_ID=0x525c2aba45f66987217323e8a05ea400c65d06dc
```

## Interacting with the Contract

Now that the contract is deployed, you can interact with it using foundry's `cast` command.

#### Creating a counter

To create a new counter, use the following command:

```bash
cast send $CONTRACT_ID "create()" --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 --rpc-url http://localhost:8547
```

You should see output similar to:

```bash
blockHash            0x10be8200fb69a6c61df508cf6294b3da65edca74357237e4ef38e856fcd1f5ce
blockNumber          10
contractAddress
cumulativeGasUsed    99090
effectiveGasPrice    100000000
from                 0x3f1Eae7D46d88F08fc2F8ed27FCb2AB183EB2d0E
gasUsed              99090
logs                 [{"address":"0x525c2aba45f66987217323e8a05ea400c65d06dc","topics":["0x7445ca7ce975ec254db4c84b6d772e9c3d7c153ca8fb13d5f180e5cf000250f3","0x70a9a5599349d999ce7abadd4bb09639e9f1a364000543ad6458b8befbcdba4e"],"data":"0x","blockHash":"0x10be8200fb69a6c61df508cf6294b3da65edca74357237e4ef38e856fcd1f5ce","blockNumber":"0xa","transactionHash":"0xff2fd596fc75f46847264c259a855745461b5c4bba89446910b6d2e92b0ad7e6","transactionIndex":"0x1","logIndex":"0x0","removed":false}]
logsBloom            0x40000000000000000000000000000000000000000000000000004000000000100000000000000000000000000000000000000000000000020000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000200000000000040000000000000000000000000000000000000008000000000000000000000000000000000
root
status               1 (success)
transactionHash      0xff2fd596fc75f46847264c259a855745461b5c4bba89446910b6d2e92b0ad7e6
transactionIndex     1
type                 2
blobGasPrice
blobGasUsed
to                   0x525c2aBA45F66987217323E8a05EA400C65D06DC
gasUsedForL1         0
l1BlockNumber        0
timeboosted          false
```

From this output, it's important to extract the counter ID from the logs. In this example, the counter ID is `0x70a9a5599349d999ce7abadd4bb09639e9f1a364000543ad6458b8befbcdba4e` which is the second topic in the logs array.
We will explain what this ID is in [UID and ID](../object_model/uid_and_id.md) section. For now, you only need to know that this ID uniquely identifies the counter you've just created.

You can save the counter ID in an environment variable for easier access:

```bash
export COUNTER_ID=0x70a9a5599349d999ce7abadd4bb09639e9f1a364000543ad6458b8befbcdba4e
```


#### Reading the counter value

To read the value of the counter, use the following command:

```bash
cast call $CONTRACT_ID "read(bytes32)(uint64)" $COUNTER_ID --rpc-url http://localhost:8547
```

You should see output similar to:

```bash
1
```

Which is the initial value of the counter.

#### Incrementing the counter

To increment the counter, use the following command:

```bash
cast send $CONTRACT_ID "increment(bytes32)" $COUNTER_ID --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 --rpc-url http://localhost:8547
```

#### Setting the counter value

To set the counter to a specific value, use the following command (for example, setting it to 42):

```bash
cast send $CONTRACT_ID "setValue(bytes32,uint64)" $COUNTER_ID 42 --private-key 0xb6b15c8cb491557369f3c7d2c287b053eb229daa9c22138887752191c9520659 --rpc-url http://localhost:8547
```

> [!NOTE]
> You can perform a read operation after each of these transactions to verify that the counter value has been updated correctly.
