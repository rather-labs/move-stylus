
<a name="hello_world_delegated_counter_interface"></a>

# Module `hello_world::delegated_counter_interface`



-  [Struct `CounterCall`](#hello_world_delegated_counter_interface_CounterCall)
-  [Function `new`](#hello_world_delegated_counter_interface_new)
-  [Function `increment`](#hello_world_delegated_counter_interface_increment)
-  [Function `set_value`](#hello_world_delegated_counter_interface_set_value)


<pre><code><b>use</b> <a href="../dependencies/stylus/contract_calls.md#stylus_contract_calls">stylus::contract_calls</a>;
<b>use</b> <a href="../dependencies/stylus/event.md#stylus_event">stylus::event</a>;
<b>use</b> <a href="../dependencies/stylus/object.md#stylus_object">stylus::object</a>;
<b>use</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="hello_world_delegated_counter_interface_CounterCall"></a>

## Struct `CounterCall`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_CounterCall">CounterCall</a> <b>has</b> drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>0: <a href="../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_delegated_counter_interface_new"></a>

## Function `new`



<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_new">new</a>(configuration: <a href="../dependencies/stylus/contract_calls.md#stylus_contract_calls_CrossContractCall">stylus::contract_calls::CrossContractCall</a>): <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_CounterCall">hello_world::delegated_counter_interface::CounterCall</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_new">new</a>(configuration: CrossContractCall): <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_CounterCall">CounterCall</a> {
    <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_CounterCall">CounterCall</a>(configuration)
}
</code></pre>



</details>

<a name="hello_world_delegated_counter_interface_increment"></a>

## Function `increment`



<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_increment">increment</a>(self: &<a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_CounterCall">hello_world::delegated_counter_interface::CounterCall</a>, <a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<b>mut</b> <a href="../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>): <a href="../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallEmptyResult">stylus::contract_calls::ContractCallEmptyResult</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_increment">increment</a>(self: &<a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_CounterCall">CounterCall</a>, <a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<b>mut</b> UID): ContractCallEmptyResult;
</code></pre>



</details>

<a name="hello_world_delegated_counter_interface_set_value"></a>

## Function `set_value`



<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_set_value">set_value</a>(self: &<a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_CounterCall">hello_world::delegated_counter_interface::CounterCall</a>, <a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<b>mut</b> <a href="../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, value: u64): <a href="../dependencies/stylus/contract_calls.md#stylus_contract_calls_ContractCallEmptyResult">stylus::contract_calls::ContractCallEmptyResult</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_set_value">set_value</a>(self: &<a href="../hello_world/delegated_counter_interface.md#hello_world_delegated_counter_interface_CounterCall">CounterCall</a>, <a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<b>mut</b> UID, value: u64): ContractCallEmptyResult;
</code></pre>



</details>
