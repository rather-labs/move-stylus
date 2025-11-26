
<a name="erc20call_erc20call"></a>

# Module `erc20call::erc20call`



-  [Struct `ERC20`](#erc20call_erc20call_ERC20)
-  [Function `new`](#erc20call_erc20call_new)
-  [Function `total_supply`](#erc20call_erc20call_total_supply)
-  [Function `balance_of`](#erc20call_erc20call_balance_of)
-  [Function `transfer`](#erc20call_erc20call_transfer)
-  [Function `allowance`](#erc20call_erc20call_allowance)
-  [Function `approve`](#erc20call_erc20call_approve)
-  [Function `transfer_from`](#erc20call_erc20call_transfer_from)


<pre><code><b>use</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls">stylus::contract_calls</a>;
</code></pre>



<a name="erc20call_erc20call_ERC20"></a>

## Struct `ERC20`



<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">ERC20</a> <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>0: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="erc20call_erc20call_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_new">new</a>(configuration: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a>): <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">erc20call::erc20call::ERC20</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_new">new</a>(configuration: CrossContractCall): <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">ERC20</a> {
    <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">ERC20</a>(configuration)
}
</code></pre>



</details>

<a name="erc20call_erc20call_total_supply"></a>

## Function `total_supply`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_total_supply">total_supply</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">erc20call::erc20call::ERC20</a>): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">stylus::contract_calls::ContractCallResult</a>&lt;u256&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_total_supply">total_supply</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">ERC20</a>): ContractCallResult&lt;u256&gt;;
</code></pre>



</details>

<a name="erc20call_erc20call_balance_of"></a>

## Function `balance_of`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_balance_of">balance_of</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">erc20call::erc20call::ERC20</a>, account: <b>address</b>): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">stylus::contract_calls::ContractCallResult</a>&lt;u256&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_balance_of">balance_of</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">ERC20</a>, account: <b>address</b>): ContractCallResult&lt;u256&gt;;
</code></pre>



</details>

<a name="erc20call_erc20call_transfer"></a>

## Function `transfer`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_transfer">transfer</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">erc20call::erc20call::ERC20</a>, account: <b>address</b>, amount: u256): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">stylus::contract_calls::ContractCallResult</a>&lt;bool&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_transfer">transfer</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">ERC20</a>, account: <b>address</b>, amount: u256): ContractCallResult&lt;bool&gt;;
</code></pre>



</details>

<a name="erc20call_erc20call_allowance"></a>

## Function `allowance`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_allowance">allowance</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">erc20call::erc20call::ERC20</a>, owner: <b>address</b>, spender: <b>address</b>): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">stylus::contract_calls::ContractCallResult</a>&lt;u256&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_allowance">allowance</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">ERC20</a>, owner: <b>address</b>, spender: <b>address</b>): ContractCallResult&lt;u256&gt;;
</code></pre>



</details>

<a name="erc20call_erc20call_approve"></a>

## Function `approve`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_approve">approve</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">erc20call::erc20call::ERC20</a>, spender: <b>address</b>, amount: u256): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">stylus::contract_calls::ContractCallResult</a>&lt;bool&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_approve">approve</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">ERC20</a>, spender: <b>address</b>, amount: u256): ContractCallResult&lt;bool&gt;;
</code></pre>



</details>

<a name="erc20call_erc20call_transfer_from"></a>

## Function `transfer_from`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_transfer_from">transfer_from</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">erc20call::erc20call::ERC20</a>, sender: <b>address</b>, recipient: <b>address</b>, amount: u256): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">stylus::contract_calls::ContractCallResult</a>&lt;bool&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_transfer_from">transfer_from</a>(self: &<a href="../../dependencies/erc20call/erc20call.md#erc20call_erc20call_ERC20">ERC20</a>, sender: <b>address</b>, recipient: <b>address</b>, amount: u256): ContractCallResult&lt;bool&gt;;
</code></pre>



</details>
