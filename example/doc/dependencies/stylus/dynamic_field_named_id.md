
<a name="stylus_dynamic_field_named_id"></a>

# Module `stylus::dynamic_field_named_id`



-  [Function `add`](#stylus_dynamic_field_named_id_add)
-  [Function `borrow`](#stylus_dynamic_field_named_id_borrow)
-  [Function `borrow_mut`](#stylus_dynamic_field_named_id_borrow_mut)
-  [Function `remove`](#stylus_dynamic_field_named_id_remove)
-  [Function `exists_`](#stylus_dynamic_field_named_id_exists_)
-  [Function `remove_if_exists`](#stylus_dynamic_field_named_id_remove_if_exists)
-  [Function `exists_with_type`](#stylus_dynamic_field_named_id_exists_with_type)


<pre><code><b>use</b> <a href="../../dependencies/std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../../dependencies/std/vector.md#std_vector">std::vector</a>;
<b>use</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field">stylus::dynamic_field</a>;
<b>use</b> <a href="../../dependencies/stylus/event.md#stylus_event">stylus::event</a>;
<b>use</b> <a href="../../dependencies/stylus/object.md#stylus_object">stylus::object</a>;
<b>use</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="stylus_dynamic_field_named_id_add"></a>

## Function `add`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_add">add</a>&lt;NId: key, Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;NId&gt;, name: Name, value: Value)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_add">add</a>&lt;NId: key, Name: <b>copy</b> + drop + store, Value: store&gt;(
    object: &<b>mut</b> NamedId&lt;NId&gt;,
    name: Name,
    value: Value,
) {
    <b>let</b> uid = object.as_uid_mut();
    dynamic_field::add(uid, name, value);
}
</code></pre>



</details>

<a name="stylus_dynamic_field_named_id_borrow"></a>

## Function `borrow`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_borrow">borrow</a>&lt;NId: key, Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;NId&gt;, name: Name): &Value
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_borrow">borrow</a>&lt;NId: key, Name: <b>copy</b> + drop + store, Value: store&gt;(
    object: &NamedId&lt;NId&gt;,
    name: Name
): &Value {
    <b>let</b> uid = object.as_uid();
    dynamic_field::borrow(uid, name)
}
</code></pre>



</details>

<a name="stylus_dynamic_field_named_id_borrow_mut"></a>

## Function `borrow_mut`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_borrow_mut">borrow_mut</a>&lt;NId: key, Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;NId&gt;, name: Name): &<b>mut</b> Value
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_borrow_mut">borrow_mut</a>&lt;NId: key, Name: <b>copy</b> + drop + store, Value: store&gt;(
    object: &<b>mut</b> NamedId&lt;NId&gt;,
    name: Name,
): &<b>mut</b> Value {
    <b>let</b> uid = object.as_uid_mut();
    dynamic_field::borrow_mut(uid, name)
}
</code></pre>



</details>

<a name="stylus_dynamic_field_named_id_remove"></a>

## Function `remove`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_remove">remove</a>&lt;NId: key, Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;NId&gt;, name: Name): Value
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_remove">remove</a>&lt;NId: key, Name: <b>copy</b> + drop + store, Value: store&gt;(
    object: &<b>mut</b> NamedId&lt;NId&gt;,
    name: Name
): Value {
    <b>let</b> uid = object.as_uid_mut();
    dynamic_field::remove(uid, name)
}
</code></pre>



</details>

<a name="stylus_dynamic_field_named_id_exists_"></a>

## Function `exists_`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_exists_">exists_</a>&lt;NId: key, Name: <b>copy</b>, drop, store&gt;(object: &<a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;NId&gt;, name: Name): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_exists_">exists_</a>&lt;NId: key, Name: <b>copy</b> + drop + store&gt;(
    object: &NamedId&lt;NId&gt;,
    name: Name
): bool {
    <b>let</b> uid = object.as_uid();
    dynamic_field::exists_(uid, name)
}
</code></pre>



</details>

<a name="stylus_dynamic_field_named_id_remove_if_exists"></a>

## Function `remove_if_exists`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_remove_if_exists">remove_if_exists</a>&lt;NId: key, Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;NId&gt;, name: Name): <a href="../../dependencies/std/option.md#std_option_Option">std::option::Option</a>&lt;Value&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_remove_if_exists">remove_if_exists</a>&lt;NId: key, Name: <b>copy</b> + drop + store, Value: store&gt;(
    object: &<b>mut</b> NamedId&lt;NId&gt;,
    name: Name,
): Option&lt;Value&gt; {
    <b>let</b> uid = object.as_uid_mut();
    dynamic_field::remove_if_exists(uid, name)
}
</code></pre>



</details>

<a name="stylus_dynamic_field_named_id_exists_with_type"></a>

## Function `exists_with_type`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_exists_with_type">exists_with_type</a>&lt;NId: key, Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;NId&gt;, name: Name): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field_named_id.md#stylus_dynamic_field_named_id_exists_with_type">exists_with_type</a>&lt;NId: key, Name: <b>copy</b> + drop + store, Value: store&gt;(
    object: &NamedId&lt;NId&gt;,
    name: Name,
): bool {
    <b>let</b> uid = object.as_uid();
    dynamic_field::exists_with_type&lt;Name, Value&gt;(uid, name)
}
</code></pre>



</details>
