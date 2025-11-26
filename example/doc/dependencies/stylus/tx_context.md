
<a name="stylus_tx_context"></a>

# Module `stylus::tx_context`



-  [Struct `TxContext`](#stylus_tx_context_TxContext)
-  [Function `sender`](#stylus_tx_context_sender)
-  [Function `native_sender`](#stylus_tx_context_native_sender)
-  [Function `msg_value`](#stylus_tx_context_msg_value)
-  [Function `native_msg_value`](#stylus_tx_context_native_msg_value)
-  [Function `block_number`](#stylus_tx_context_block_number)
-  [Function `native_block_number`](#stylus_tx_context_native_block_number)
-  [Function `block_basefee`](#stylus_tx_context_block_basefee)
-  [Function `native_block_basefee`](#stylus_tx_context_native_block_basefee)
-  [Function `block_gas_limit`](#stylus_tx_context_block_gas_limit)
-  [Function `native_block_gas_limit`](#stylus_tx_context_native_block_gas_limit)
-  [Function `block_timestamp`](#stylus_tx_context_block_timestamp)
-  [Function `native_block_timestamp`](#stylus_tx_context_native_block_timestamp)
-  [Function `chain_id`](#stylus_tx_context_chain_id)
-  [Function `native_chain_id`](#stylus_tx_context_native_chain_id)
-  [Function `gas_price`](#stylus_tx_context_gas_price)
-  [Function `native_gas_price`](#stylus_tx_context_native_gas_price)
-  [Function `fresh_object_address`](#stylus_tx_context_fresh_object_address)
-  [Function `fresh_id`](#stylus_tx_context_fresh_id)


<pre><code></code></pre>



<a name="stylus_tx_context_TxContext"></a>

## Struct `TxContext`

Information about the transaction currently being executed.
This cannot be constructed by a transaction--it is a privileged object created by
the VM and passed in to the entrypoint of the transaction as <code>&<b>mut</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a></code>.


<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a> <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="stylus_tx_context_sender"></a>

## Function `sender`

Return the address of the user that signed the current
transaction


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_sender">sender</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_sender">sender</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a>): <b>address</b> {
    <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_sender">native_sender</a>()
}
</code></pre>



</details>

<a name="stylus_tx_context_native_sender"></a>

## Function `native_sender`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_sender">native_sender</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_sender">native_sender</a>(): <b>address</b>;
</code></pre>



</details>

<a name="stylus_tx_context_msg_value"></a>

## Function `msg_value`

Return the number of wei sent with the message


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_msg_value">msg_value</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_msg_value">msg_value</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a>): u256 {
    <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_msg_value">native_msg_value</a>()
}
</code></pre>



</details>

<a name="stylus_tx_context_native_msg_value"></a>

## Function `native_msg_value`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_msg_value">native_msg_value</a>(): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_msg_value">native_msg_value</a>(): u256;
</code></pre>



</details>

<a name="stylus_tx_context_block_number"></a>

## Function `block_number`

Return the current block's number.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_block_number">block_number</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_block_number">block_number</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a>): u64 {
    <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_number">native_block_number</a>()
}
</code></pre>



</details>

<a name="stylus_tx_context_native_block_number"></a>

## Function `native_block_number`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_number">native_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_number">native_block_number</a>(): u64;
</code></pre>



</details>

<a name="stylus_tx_context_block_basefee"></a>

## Function `block_basefee`

Return the current block's base fee (EIP-3198 and EIP-1559)


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_block_basefee">block_basefee</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_block_basefee">block_basefee</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a>): u256 {
    <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_basefee">native_block_basefee</a>()
}
</code></pre>



</details>

<a name="stylus_tx_context_native_block_basefee"></a>

## Function `native_block_basefee`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_basefee">native_block_basefee</a>(): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_basefee">native_block_basefee</a>(): u256;
</code></pre>



</details>

<a name="stylus_tx_context_block_gas_limit"></a>

## Function `block_gas_limit`

Return the current block's gas limit.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_block_gas_limit">block_gas_limit</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_block_gas_limit">block_gas_limit</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a>): u64 {
    <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_gas_limit">native_block_gas_limit</a>()
}
</code></pre>



</details>

<a name="stylus_tx_context_native_block_gas_limit"></a>

## Function `native_block_gas_limit`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_gas_limit">native_block_gas_limit</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_gas_limit">native_block_gas_limit</a>(): u64;
</code></pre>



</details>

<a name="stylus_tx_context_block_timestamp"></a>

## Function `block_timestamp`

Return the current block's timestamp as seconds since unix epoch


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_block_timestamp">block_timestamp</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_block_timestamp">block_timestamp</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a>): u64 {
    <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_timestamp">native_block_timestamp</a>()
}
</code></pre>



</details>

<a name="stylus_tx_context_native_block_timestamp"></a>

## Function `native_block_timestamp`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_timestamp">native_block_timestamp</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_block_timestamp">native_block_timestamp</a>(): u64;
</code></pre>



</details>

<a name="stylus_tx_context_chain_id"></a>

## Function `chain_id`

Return the chain ID of the current transaction.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_chain_id">chain_id</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_chain_id">chain_id</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a>): u64 {
    <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_chain_id">native_chain_id</a>()
}
</code></pre>



</details>

<a name="stylus_tx_context_native_chain_id"></a>

## Function `native_chain_id`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_chain_id">native_chain_id</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_chain_id">native_chain_id</a>(): u64;
</code></pre>



</details>

<a name="stylus_tx_context_gas_price"></a>

## Function `gas_price`

Return the gas price of the transaction


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_gas_price">gas_price</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_gas_price">gas_price</a>(_self: &<a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a>): u256 {
    <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_gas_price">native_gas_price</a>()
}
</code></pre>



</details>

<a name="stylus_tx_context_native_gas_price"></a>

## Function `native_gas_price`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_gas_price">native_gas_price</a>(): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_native_gas_price">native_gas_price</a>(): u256;
</code></pre>



</details>

<a name="stylus_tx_context_fresh_object_address"></a>

## Function `fresh_object_address`

Create an <code><b>address</b></code> that has not been used. As it is an object address, it will never
occur as the address for a user.
In other words, the generated address is a globally unique object ID.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_fresh_object_address">fresh_object_address</a>(_ctx: &<b>mut</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_fresh_object_address">fresh_object_address</a>(_ctx: &<b>mut</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">TxContext</a>): <b>address</b> {
    <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_fresh_id">fresh_id</a>()
}
</code></pre>



</details>

<a name="stylus_tx_context_fresh_id"></a>

## Function `fresh_id`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_fresh_id">fresh_id</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_fresh_id">fresh_id</a>(): <b>address</b>;
</code></pre>



</details>
