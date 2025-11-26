
<a name="stylus_object"></a>

# Module `stylus::object`



-  [Struct `ID`](#stylus_object_ID)
-  [Struct `UID`](#stylus_object_UID)
-  [Struct `NewUID`](#stylus_object_NewUID)
-  [Struct `NamedId`](#stylus_object_NamedId)
-  [Function `new`](#stylus_object_new)
-  [Function `delete`](#stylus_object_delete)
-  [Function `new_uid_from_hash`](#stylus_object_new_uid_from_hash)
-  [Function `uid_to_address`](#stylus_object_uid_to_address)
-  [Function `uid_to_inner`](#stylus_object_uid_to_inner)
-  [Function `compute_named_id`](#stylus_object_compute_named_id)
-  [Function `new_named_id`](#stylus_object_new_named_id)
-  [Function `remove`](#stylus_object_remove)
-  [Function `as_uid`](#stylus_object_as_uid)
-  [Function `as_uid_mut`](#stylus_object_as_uid_mut)


<pre><code><b>use</b> <a href="../../dependencies/stylus/event.md#stylus_event">stylus::event</a>;
<b>use</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="stylus_object_ID"></a>

## Struct `ID`

References a object ID


<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/object.md#stylus_object_ID">ID</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>bytes: <b>address</b></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="stylus_object_UID"></a>

## Struct `UID`

Globally unique IDs that define an object's ID in storage. Any object, that is a struct
with the <code>key</code> ability, must have <code>id: <a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a></code> as its first field.


<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a> <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../../dependencies/stylus/object.md#stylus_object_ID">stylus::object::ID</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="stylus_object_NewUID"></a>

## Struct `NewUID`



<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/object.md#stylus_object_NewUID">NewUID</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>uid: <b>address</b></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="stylus_object_NamedId"></a>

## Struct `NamedId`

Named IDs are used know where the object will saved in storage, so we don't depend on the
user to pass the object UID to retrieve it from storage.

This struct is an special struct managed by the compiler. The name is given by the T struct
passed as type parameter. For example:

```move
public struct TOTAL_SUPPLY has key {}

public struct TotalSupply has key {
id: NamedId<TOTAL_SUPPLY>,
total: u256,
}
```

<code><a href="../../dependencies/stylus/object.md#stylus_object_NamedId">NamedId</a></code>'s can only be used in one struct. Detecting the same NamedId in two different
structs will result in a compilation error.


<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">NamedId</a>&lt;<b>phantom</b> T: key&gt; <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../../dependencies/stylus/object.md#stylus_object_ID">stylus::object::ID</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="stylus_object_new"></a>

## Function `new`

Creates a new <code><a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a></code>, which must be stored in an object's <code>id</code> field.
This is the only way to create <code><a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a></code>s.

Each time a new <code><a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a></code> is created, an event is emitted on topic 0.
This allows the transaction caller to capture and persist it for later
reference to the object associated with that <code><a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_new">new</a>(ctx: &<b>mut</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_new">new</a>(ctx: &<b>mut</b> TxContext): <a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a> {
    <b>let</b> res = <a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a> { id: <a href="../../dependencies/stylus/object.md#stylus_object_ID">ID</a> { bytes: ctx.fresh_object_address() } };
    emit(<a href="../../dependencies/stylus/object.md#stylus_object_NewUID">NewUID</a> { uid: res.to_address() });
    res
}
</code></pre>



</details>

<a name="stylus_object_delete"></a>

## Function `delete`

Deletes the object from the storage.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_delete">delete</a>(id: <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_delete">delete</a>(id: <a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a>);
</code></pre>



</details>

<a name="stylus_object_new_uid_from_hash"></a>

## Function `new_uid_from_hash`

Generate a new UID specifically used for creating a UID from a hash


<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_new_uid_from_hash">new_uid_from_hash</a>(bytes: <b>address</b>): <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_new_uid_from_hash">new_uid_from_hash</a>(bytes: <b>address</b>): <a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a> {
    <a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a> { id: <a href="../../dependencies/stylus/object.md#stylus_object_ID">ID</a> { bytes } }
}
</code></pre>



</details>

<a name="stylus_object_uid_to_address"></a>

## Function `uid_to_address`

Get the inner bytes of <code>id</code> as an address.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_uid_to_address">uid_to_address</a>(uid: &<a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_uid_to_address">uid_to_address</a>(uid: &<a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a>): <b>address</b> {
    uid.id.bytes
}
</code></pre>



</details>

<a name="stylus_object_uid_to_inner"></a>

## Function `uid_to_inner`

Get the raw bytes of a <code>uid</code>'s inner <code><a href="../../dependencies/stylus/object.md#stylus_object_ID">ID</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_uid_to_inner">uid_to_inner</a>(uid: &<a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>): <a href="../../dependencies/stylus/object.md#stylus_object_ID">stylus::object::ID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_uid_to_inner">uid_to_inner</a>(uid: &<a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a>): <a href="../../dependencies/stylus/object.md#stylus_object_ID">ID</a> {
    uid.id
}
</code></pre>



</details>

<a name="stylus_object_compute_named_id"></a>

## Function `compute_named_id`



<pre><code><b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_compute_named_id">compute_named_id</a>&lt;T: key&gt;(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_compute_named_id">compute_named_id</a>&lt;T: key&gt;(): <b>address</b>;
</code></pre>



</details>

<a name="stylus_object_new_named_id"></a>

## Function `new_named_id`



<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_new_named_id">new_named_id</a>&lt;T: key&gt;(): <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_new_named_id">new_named_id</a>&lt;T: key&gt;(): <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">NamedId</a>&lt;T&gt; {
    <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">NamedId</a> { id: <a href="../../dependencies/stylus/object.md#stylus_object_ID">ID</a> { bytes: <a href="../../dependencies/stylus/object.md#stylus_object_compute_named_id">compute_named_id</a>&lt;T&gt;() } }
}
</code></pre>



</details>

<a name="stylus_object_remove"></a>

## Function `remove`

Deletes the object with a <code><a href="../../dependencies/stylus/object.md#stylus_object_NamedId">NamedId</a></code> from the storage.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_remove">remove</a>&lt;T: key&gt;(id: <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_remove">remove</a>&lt;T: key&gt;(id: <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">NamedId</a>&lt;T&gt;);
</code></pre>



</details>

<a name="stylus_object_as_uid"></a>

## Function `as_uid`



<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_as_uid">as_uid</a>&lt;T: key&gt;(named_id: &<a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;T&gt;): &<a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_as_uid">as_uid</a>&lt;T: key&gt;(named_id: &<a href="../../dependencies/stylus/object.md#stylus_object_NamedId">NamedId</a>&lt;T&gt;): &<a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a>;
</code></pre>



</details>

<a name="stylus_object_as_uid_mut"></a>

## Function `as_uid_mut`



<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_as_uid_mut">as_uid_mut</a>&lt;T: key&gt;(named_id: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">stylus::object::NamedId</a>&lt;T&gt;): &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/object.md#stylus_object_as_uid_mut">as_uid_mut</a>&lt;T: key&gt;(named_id: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_NamedId">NamedId</a>&lt;T&gt;): &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_UID">UID</a>;
</code></pre>



</details>
