
<a name="hello_world_counter_named_id"></a>

# Module `hello_world::counter_named_id`



-  [Struct `COUNTER_`](#hello_world_counter_named_id_COUNTER_)
-  [Struct `Counter`](#hello_world_counter_named_id_Counter)
-  [Function `create`](#hello_world_counter_named_id_create)
-  [Function `increment`](#hello_world_counter_named_id_increment)
-  [Function `read`](#hello_world_counter_named_id_read)
-  [Function `set_value`](#hello_world_counter_named_id_set_value)


<pre><code><b>use</b> <a href="../dependencies/stylus/event.md#stylus_event">stylus::event</a>;
<b>use</b> <a href="../dependencies/stylus/object.md#stylus_object">stylus::object</a>;
<b>use</b> <a href="../dependencies/stylus/transfer.md#stylus_transfer">stylus::transfer</a>;
<b>use</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="hello_world_counter_named_id_COUNTER_"></a>

## Struct `COUNTER_`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_COUNTER_">COUNTER_</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="hello_world_counter_named_id_Counter"></a>

## Struct `Counter`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_Counter">Counter</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;<a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_COUNTER_">hello_world::counter_named_id::COUNTER_</a>&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>owner: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>value: u64</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_counter_named_id_create"></a>

## Function `create`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_create">create</a>(ctx: &<b>mut</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_create">create</a>(ctx: &<b>mut</b> TxContext) {
  transfer::share_object(<a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_Counter">Counter</a> {
    id: object::new_named_id&lt;<a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_COUNTER_">COUNTER_</a>&gt;(),
    owner: ctx.sender(),
    value: 25
  });
}
</code></pre>



</details>

<a name="hello_world_counter_named_id_increment"></a>

## Function `increment`

Increment a counter by 1.


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_increment">increment</a>(<a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<b>mut</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_Counter">hello_world::counter_named_id::Counter</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_increment">increment</a>(<a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<b>mut</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_Counter">Counter</a>) {
    <a href="../hello_world/counter.md#hello_world_counter">counter</a>.value = <a href="../hello_world/counter.md#hello_world_counter">counter</a>.value + 1;
}
</code></pre>



</details>

<a name="hello_world_counter_named_id_read"></a>

## Function `read`

Read counter.


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_read">read</a>(<a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_Counter">hello_world::counter_named_id::Counter</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_read">read</a>(<a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_Counter">Counter</a>): u64 {
    <a href="../hello_world/counter.md#hello_world_counter">counter</a>.value
}
</code></pre>



</details>

<a name="hello_world_counter_named_id_set_value"></a>

## Function `set_value`

Set value (only runnable by the Counter owner)


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_set_value">set_value</a>(<a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<b>mut</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_Counter">hello_world::counter_named_id::Counter</a>, value: u64, ctx: &<a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_set_value">set_value</a>(<a href="../hello_world/counter.md#hello_world_counter">counter</a>: &<b>mut</b> <a href="../hello_world/counter_named_id.md#hello_world_counter_named_id_Counter">Counter</a>, value: u64, ctx: &TxContext) {
    <b>assert</b>!(<a href="../hello_world/counter.md#hello_world_counter">counter</a>.owner == ctx.sender(), 0);
    <a href="../hello_world/counter.md#hello_world_counter">counter</a>.value = value;
}
</code></pre>



</details>
