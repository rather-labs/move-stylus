
<a name="stylus_contract_calls"></a>

# Module `stylus::contract_calls`



-  [Struct `CrossContractCall`](#stylus_contract_calls_CrossContractCall)
-  [Struct `ContractCallResult`](#stylus_contract_calls_ContractCallResult)
-  [Struct `ContractCallEmptyResult`](#stylus_contract_calls_ContractCallEmptyResult)
-  [Constants](#@Constants_0)
-  [Function `new`](#stylus_contract_calls_new)
-  [Function `gas`](#stylus_contract_calls_gas)
-  [Function `value`](#stylus_contract_calls_value)
-  [Function `delegate`](#stylus_contract_calls_delegate)
-  [Function `succeded`](#stylus_contract_calls_succeded)
-  [Function `get_result`](#stylus_contract_calls_get_result)
-  [Function `empty_result_succeded`](#stylus_contract_calls_empty_result_succeded)


<pre><code></code></pre>



<a name="stylus_contract_calls_CrossContractCall"></a>

## Struct `CrossContractCall`



<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">CrossContractCall</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>contract_address: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code><a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_delegate">delegate</a>: bool</code>
</dt>
<dd>
</dd>
<dt>
<code><a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_gas">gas</a>: u64</code>
</dt>
<dd>
</dd>
<dt>
<code><a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_value">value</a>: u256</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="stylus_contract_calls_ContractCallResult"></a>

## Struct `ContractCallResult`



<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">ContractCallResult</a>&lt;RESULT&gt; <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>code: u8</code>
</dt>
<dd>
</dd>
<dt>
<code>result: RESULT</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="stylus_contract_calls_ContractCallEmptyResult"></a>

## Struct `ContractCallEmptyResult`



<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallEmptyResult">ContractCallEmptyResult</a> <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>code: u8</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="stylus_contract_calls_ECallFailed"></a>



<pre><code><b>const</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ECallFailed">ECallFailed</a>: u64 = 101;
</code></pre>



<a name="stylus_contract_calls_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_new">new</a>(contract_address: <b>address</b>): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_new">new</a>(contract_address: <b>address</b>): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">CrossContractCall</a> {
    <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">CrossContractCall</a> {
        contract_address,
        <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_delegate">delegate</a>: <b>false</b>,
        <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_gas">gas</a>: 0xffffffffffffffff,
        <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_value">value</a>: 0,
    }
}
</code></pre>



</details>

<a name="stylus_contract_calls_gas"></a>

## Function `gas`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_gas">gas</a>(self: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a>, <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_gas">gas</a>: u64): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_gas">gas</a>(<b>mut</b> self: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">CrossContractCall</a>, <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_gas">gas</a>: u64): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">CrossContractCall</a> {
    self.<a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_gas">gas</a> = <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_gas">gas</a>;
    self
}
</code></pre>



</details>

<a name="stylus_contract_calls_value"></a>

## Function `value`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_value">value</a>(self: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a>, <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_value">value</a>: u256): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_value">value</a>(<b>mut</b> self: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">CrossContractCall</a>, <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_value">value</a>: u256): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">CrossContractCall</a> {
    self.<a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_value">value</a> = <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_value">value</a>;
    self
}
</code></pre>



</details>

<a name="stylus_contract_calls_delegate"></a>

## Function `delegate`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_delegate">delegate</a>(self: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a>): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_delegate">delegate</a>(<b>mut</b> self: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">CrossContractCall</a>): <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">CrossContractCall</a> {
    self.<a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_delegate">delegate</a> = <b>true</b>;
    self
}
</code></pre>



</details>

<a name="stylus_contract_calls_succeded"></a>

## Function `succeded`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_succeded">succeded</a>&lt;T&gt;(self: &<a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">stylus::contract_calls::ContractCallResult</a>&lt;T&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_succeded">succeded</a>&lt;T&gt;(self: &<a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">ContractCallResult</a>&lt;T&gt;): bool {
    self.code == 0
}
</code></pre>



</details>

<a name="stylus_contract_calls_get_result"></a>

## Function `get_result`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_get_result">get_result</a>&lt;T&gt;(self: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">stylus::contract_calls::ContractCallResult</a>&lt;T&gt;): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_get_result">get_result</a>&lt;T&gt;(self: <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">ContractCallResult</a>&lt;T&gt;): T {
    <b>let</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallResult">ContractCallResult</a> { code, result } = self;
    <b>assert</b>!(code == 0, <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ECallFailed">ECallFailed</a>);
    result
}
</code></pre>



</details>

<a name="stylus_contract_calls_empty_result_succeded"></a>

## Function `empty_result_succeded`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_empty_result_succeded">empty_result_succeded</a>(self: &<a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallEmptyResult">stylus::contract_calls::ContractCallEmptyResult</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_empty_result_succeded">empty_result_succeded</a>(self: &<a href="../../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallEmptyResult">ContractCallEmptyResult</a>): bool {
    self.code == 0
}
</code></pre>



</details>
