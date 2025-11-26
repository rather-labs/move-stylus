
<a name="hello_world_hello_world_2"></a>

# Module `hello_world::hello_world_2`



-  [Struct `TestEvent1`](#hello_world_hello_world_2_TestEvent1)
-  [Struct `TestEvent2`](#hello_world_hello_world_2_TestEvent2)
-  [Struct `NestedStruct1`](#hello_world_hello_world_2_NestedStruct1)
-  [Struct `NestedStruct2`](#hello_world_hello_world_2_NestedStruct2)
-  [Struct `TestEvent3`](#hello_world_hello_world_2_TestEvent3)
-  [Function `echo_with_generic_function_u16`](#hello_world_hello_world_2_echo_with_generic_function_u16)
-  [Function `echo_with_generic_function_vec32`](#hello_world_hello_world_2_echo_with_generic_function_vec32)
-  [Function `echo_with_generic_function_u16_vec32`](#hello_world_hello_world_2_echo_with_generic_function_u16_vec32)
-  [Function `echo_with_generic_function_address_vec128`](#hello_world_hello_world_2_echo_with_generic_function_address_vec128)
-  [Function `get_fresh_object_address`](#hello_world_hello_world_2_get_fresh_object_address)
-  [Function `get_unique_ids`](#hello_world_hello_world_2_get_unique_ids)
-  [Function `get_unique_id`](#hello_world_hello_world_2_get_unique_id)
-  [Function `emit_test_event1`](#hello_world_hello_world_2_emit_test_event1)
-  [Function `emit_test_event2`](#hello_world_hello_world_2_emit_test_event2)
-  [Function `emit_test_event3`](#hello_world_hello_world_2_emit_test_event3)
-  [Function `test_stack_1`](#hello_world_hello_world_2_test_stack_1)
-  [Function `test_stack_2`](#hello_world_hello_world_2_test_stack_2)
-  [Function `test_stack_3`](#hello_world_hello_world_2_test_stack_3)


<pre><code><b>use</b> <a href="../hello_world/another_mod.md#hello_world_another_mod">hello_world::another_mod</a>;
<b>use</b> <a href="../hello_world/other_mod.md#hello_world_other_mod">hello_world::other_mod</a>;
<b>use</b> <a href="../hello_world/stack.md#hello_world_stack">hello_world::stack</a>;
<b>use</b> <a href="../dependencies/std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../dependencies/std/vector.md#std_vector">std::vector</a>;
<b>use</b> <a href="../dependencies/stylus/event.md#stylus_event">stylus::event</a>;
<b>use</b> <a href="../dependencies/stylus/object.md#stylus_object">stylus::object</a>;
<b>use</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="hello_world_hello_world_2_TestEvent1"></a>

## Struct `TestEvent1`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_TestEvent1">TestEvent1</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>n: u32</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_hello_world_2_TestEvent2"></a>

## Struct `TestEvent2`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_TestEvent2">TestEvent2</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>a: u32</code>
</dt>
<dd>
</dd>
<dt>
<code>b: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>c: u128</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_hello_world_2_NestedStruct1"></a>

## Struct `NestedStruct1`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_NestedStruct1">NestedStruct1</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>n: u32</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_hello_world_2_NestedStruct2"></a>

## Struct `NestedStruct2`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_NestedStruct2">NestedStruct2</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>a: u32</code>
</dt>
<dd>
</dd>
<dt>
<code>b: vector&lt;u8&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>c: u128</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_hello_world_2_TestEvent3"></a>

## Struct `TestEvent3`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_TestEvent3">TestEvent3</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>a: <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_NestedStruct1">hello_world::hello_world_2::NestedStruct1</a></code>
</dt>
<dd>
</dd>
<dt>
<code>b: <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_NestedStruct2">hello_world::hello_world_2::NestedStruct2</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_hello_world_2_echo_with_generic_function_u16"></a>

## Function `echo_with_generic_function_u16`



<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_echo_with_generic_function_u16">echo_with_generic_function_u16</a>(x: u16): u16
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_echo_with_generic_function_u16">echo_with_generic_function_u16</a>(x: u16): u16 {
    generic_identity(x)
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_echo_with_generic_function_vec32"></a>

## Function `echo_with_generic_function_vec32`



<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_echo_with_generic_function_vec32">echo_with_generic_function_vec32</a>(x: vector&lt;u32&gt;): vector&lt;u32&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_echo_with_generic_function_vec32">echo_with_generic_function_vec32</a>(x: vector&lt;u32&gt;): vector&lt;u32&gt; {
    generic_identity(x)
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_echo_with_generic_function_u16_vec32"></a>

## Function `echo_with_generic_function_u16_vec32`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_echo_with_generic_function_u16_vec32">echo_with_generic_function_u16_vec32</a>(x: u16, y: vector&lt;u32&gt;): (u16, vector&lt;u32&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_echo_with_generic_function_u16_vec32">echo_with_generic_function_u16_vec32</a>(x: u16, y: vector&lt;u32&gt;): (u16, vector&lt;u32&gt;) {
    generic_identity_two_types(x, y)
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_echo_with_generic_function_address_vec128"></a>

## Function `echo_with_generic_function_address_vec128`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_echo_with_generic_function_address_vec128">echo_with_generic_function_address_vec128</a>(x: <b>address</b>, y: vector&lt;u128&gt;): (<b>address</b>, vector&lt;u128&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_echo_with_generic_function_address_vec128">echo_with_generic_function_address_vec128</a>(x: <b>address</b>, y: vector&lt;u128&gt;): (<b>address</b>, vector&lt;u128&gt;) {
    generic_identity_two_types(x, y)
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_get_fresh_object_address"></a>

## Function `get_fresh_object_address`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_get_fresh_object_address">get_fresh_object_address</a>(ctx: &<b>mut</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_get_fresh_object_address">get_fresh_object_address</a>(ctx: &<b>mut</b> TxContext): <b>address</b> {
    ctx.fresh_object_address()
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_get_unique_ids"></a>

## Function `get_unique_ids`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_get_unique_ids">get_unique_ids</a>(ctx: &<b>mut</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): (<a href="../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, <a href="../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, <a href="../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_get_unique_ids">get_unique_ids</a>(ctx: &<b>mut</b> TxContext): (UID, UID, UID) {
    (
        object::new(ctx),
        object::new(ctx),
        object::new(ctx),
    )
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_get_unique_id"></a>

## Function `get_unique_id`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_get_unique_id">get_unique_id</a>(ctx: &<b>mut</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): <a href="../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_get_unique_id">get_unique_id</a>(ctx: &<b>mut</b> TxContext): UID {
    object::new(ctx)
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_emit_test_event1"></a>

## Function `emit_test_event1`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_emit_test_event1">emit_test_event1</a>(n: u32)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_emit_test_event1">emit_test_event1</a>(n: u32) {
    emit(<a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_TestEvent1">TestEvent1</a> { n });
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_emit_test_event2"></a>

## Function `emit_test_event2`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_emit_test_event2">emit_test_event2</a>(a: u32, b: vector&lt;u8&gt;, c: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_emit_test_event2">emit_test_event2</a>(a: u32, b: vector&lt;u8&gt;, c: u128) {
    emit(<a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_TestEvent2">TestEvent2</a> { a, b, c });
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_emit_test_event3"></a>

## Function `emit_test_event3`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_emit_test_event3">emit_test_event3</a>(n: u32, a: u32, b: vector&lt;u8&gt;, c: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_emit_test_event3">emit_test_event3</a>(n: u32, a: u32, b: vector&lt;u8&gt;, c: u128) {
    emit(<a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_TestEvent3">TestEvent3</a> { a: <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_NestedStruct1">NestedStruct1</a> { n }, b: <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_NestedStruct2">NestedStruct2</a> { a, b, c } });
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_test_stack_1"></a>

## Function `test_stack_1`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_test_stack_1">test_stack_1</a>(): (<a href="../hello_world/stack.md#hello_world_stack_Stack">hello_world::stack::Stack</a>&lt;u32&gt;, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_test_stack_1">test_stack_1</a>(): (Stack&lt;u32&gt;, u64) {
    <b>let</b> <b>mut</b> s = <a href="../hello_world/stack.md#hello_world_stack_new">stack::new</a>(vector[1, 2, 3]);
    s.push_back(5);
    s.push_back(6);
    (s, s.size())
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_test_stack_2"></a>

## Function `test_stack_2`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_test_stack_2">test_stack_2</a>(): (<a href="../hello_world/stack.md#hello_world_stack_Stack">hello_world::stack::Stack</a>&lt;u32&gt;, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_test_stack_2">test_stack_2</a>(): (Stack&lt;u32&gt;, u64){
    <b>let</b> <b>mut</b> s = <a href="../hello_world/stack.md#hello_world_stack_new">stack::new</a>(vector[]);
    s.push_back(5);
    s.push_back(6);
    (s, s.size())
}
</code></pre>



</details>

<a name="hello_world_hello_world_2_test_stack_3"></a>

## Function `test_stack_3`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_test_stack_3">test_stack_3</a>(): (<a href="../hello_world/stack.md#hello_world_stack_Stack">hello_world::stack::Stack</a>&lt;u32&gt;, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world_2.md#hello_world_hello_world_2_test_stack_3">test_stack_3</a>(): (Stack&lt;u32&gt;, u64){
    <b>let</b> <b>mut</b> s = <a href="../hello_world/stack.md#hello_world_stack_new">stack::new</a>(vector[3,1,4,1,5]);
    s.push_back(5);
    s.push_back(6);
    s.pop_back();
    s.pop_back();
    (s, s.size())
}
</code></pre>



</details>
