
<a name="hello_world_cross_contract_call"></a>

# Module `hello_world::cross_contract_call`



-  [Function `balance_of_erc20`](#hello_world_cross_contract_call_balance_of_erc20)
-  [Function `total_supply`](#hello_world_cross_contract_call_total_supply)
-  [Function `transfer_from_erc20`](#hello_world_cross_contract_call_transfer_from_erc20)


<pre><code><b>use</b> <a href="../dependencies/erc20call/erc20call.md#erc20call_erc20call">erc20call::erc20call</a>;
<b>use</b> <a href="../dependencies/stylus/contract_calls.md#stylus_contract_calls">stylus::contract_calls</a>;
</code></pre>



<a name="hello_world_cross_contract_call_balance_of_erc20"></a>

## Function `balance_of_erc20`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/cross_contract_call.md#hello_world_cross_contract_call_balance_of_erc20">balance_of_erc20</a>(erc20_address: <b>address</b>, balance_address: <b>address</b>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/cross_contract_call.md#hello_world_cross_contract_call_balance_of_erc20">balance_of_erc20</a>(erc20_address: <b>address</b>, balance_address: <b>address</b>): u256 {
    <b>let</b> <a href="../hello_world/erc20.md#hello_world_erc20">erc20</a> = erc20call::new(ccc::new(erc20_address));
    <a href="../hello_world/erc20.md#hello_world_erc20">erc20</a>.balance_of(balance_address).get_result()
}
</code></pre>



</details>

<a name="hello_world_cross_contract_call_total_supply"></a>

## Function `total_supply`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/cross_contract_call.md#hello_world_cross_contract_call_total_supply">total_supply</a>(erc20_address: <b>address</b>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/cross_contract_call.md#hello_world_cross_contract_call_total_supply">total_supply</a>(erc20_address: <b>address</b>): u256 {
    <b>let</b> <a href="../hello_world/erc20.md#hello_world_erc20">erc20</a> = erc20call::new(ccc::new(erc20_address));
    <a href="../hello_world/erc20.md#hello_world_erc20">erc20</a>.<a href="../hello_world/cross_contract_call.md#hello_world_cross_contract_call_total_supply">total_supply</a>().get_result()
}
</code></pre>



</details>

<a name="hello_world_cross_contract_call_transfer_from_erc20"></a>

## Function `transfer_from_erc20`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/cross_contract_call.md#hello_world_cross_contract_call_transfer_from_erc20">transfer_from_erc20</a>(erc20_address: <b>address</b>, sender: <b>address</b>, recipient: <b>address</b>, amount: u256): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/cross_contract_call.md#hello_world_cross_contract_call_transfer_from_erc20">transfer_from_erc20</a>(
    erc20_address: <b>address</b>,
    sender: <b>address</b>,
    recipient: <b>address</b>,
    amount: u256,
): bool {
    <b>let</b> <a href="../hello_world/erc20.md#hello_world_erc20">erc20</a> = erc20call::new(ccc::new(erc20_address));
    <a href="../hello_world/erc20.md#hello_world_erc20">erc20</a>.transfer_from(sender, recipient, amount).get_result()
}
</code></pre>



</details>
