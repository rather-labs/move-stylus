
<a name="stylus_dynamic_field"></a>

# Module `stylus::dynamic_field`

In addition to the fields declared in its type definition, a Sui object can have dynamic fields
that can be added after the object has been constructed. Unlike ordinary field names
(which are always statically declared identifiers) a dynamic field name can be any value with
the <code><b>copy</b></code>, <code>drop</code>, and <code>store</code> abilities, e.g. an integer, a boolean, or a string.
This gives Sui programmers the flexibility to extend objects on-the-fly, and it also serves as a
building block for core collection types


-  [Struct `Field`](#stylus_dynamic_field_Field)
-  [Constants](#@Constants_0)
-  [Function `add`](#stylus_dynamic_field_add)
-  [Function `borrow`](#stylus_dynamic_field_borrow)
-  [Function `borrow_mut`](#stylus_dynamic_field_borrow_mut)
-  [Function `remove`](#stylus_dynamic_field_remove)
-  [Function `exists_`](#stylus_dynamic_field_exists_)
-  [Function `remove_if_exists`](#stylus_dynamic_field_remove_if_exists)
-  [Function `exists_with_type`](#stylus_dynamic_field_exists_with_type)
-  [Function `hash_type_and_key`](#stylus_dynamic_field_hash_type_and_key)
-  [Function `add_child_object`](#stylus_dynamic_field_add_child_object)
-  [Function `borrow_child_object`](#stylus_dynamic_field_borrow_child_object)
-  [Function `borrow_child_object_mut`](#stylus_dynamic_field_borrow_child_object_mut)
-  [Function `remove_child_object`](#stylus_dynamic_field_remove_child_object)
-  [Function `has_child_object`](#stylus_dynamic_field_has_child_object)
-  [Function `has_child_object_with_ty`](#stylus_dynamic_field_has_child_object_with_ty)


<pre><code><b>use</b> <a href="../../dependencies/std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../../dependencies/std/vector.md#std_vector">std::vector</a>;
<b>use</b> <a href="../../dependencies/stylus/event.md#stylus_event">stylus::event</a>;
<b>use</b> <a href="../../dependencies/stylus/object.md#stylus_object">stylus::object</a>;
<b>use</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="stylus_dynamic_field_Field"></a>

## Struct `Field`

Internal object used for storing the field and value


<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_Field">Field</a>&lt;Name: <b>copy</b>, drop, store, Value: store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a></code>
</dt>
<dd>
 Determined by the hash of the object ID, the field name value and it's type,
 i.e. hash(parent.id || name || Name)
</dd>
<dt>
<code>name: Name</code>
</dt>
<dd>
 The value for the name of this field
</dd>
<dt>
<code>value: Value</code>
</dt>
<dd>
 The value bound to this field
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="stylus_dynamic_field_EBCSSerializationFailure"></a>

Failed to serialize the field's name


<pre><code><b>const</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EBCSSerializationFailure">EBCSSerializationFailure</a>: u64 = 3;
</code></pre>



<a name="stylus_dynamic_field_EFieldAlreadyExists"></a>

The object already has a dynamic field with this name (with the value and type specified)


<pre><code><b>const</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldAlreadyExists">EFieldAlreadyExists</a>: u64 = 0;
</code></pre>



<a name="stylus_dynamic_field_EFieldDoesNotExist"></a>

Cannot load dynamic field.
The object does not have a dynamic field with this name (with the value and type specified)


<pre><code><b>const</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldDoesNotExist">EFieldDoesNotExist</a>: u64 = 1;
</code></pre>



<a name="stylus_dynamic_field_EFieldTypeMismatch"></a>

The object has a field with that name, but the value type does not match


<pre><code><b>const</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldTypeMismatch">EFieldTypeMismatch</a>: u64 = 2;
</code></pre>



<a name="stylus_dynamic_field_ESharedObjectOperationNotSupported"></a>

The object added as a dynamic field was previously a shared object


<pre><code><b>const</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_ESharedObjectOperationNotSupported">ESharedObjectOperationNotSupported</a>: u64 = 4;
</code></pre>



<a name="stylus_dynamic_field_add"></a>

## Function `add`

Adds a dynamic field to the object <code>object: &<b>mut</b> UID</code> at field specified by <code>name: Name</code>.
Aborts with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldAlreadyExists">EFieldAlreadyExists</a></code> if the object already has that field with that name.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_add">add</a>&lt;Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, name: Name, value: Value)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_add">add</a>&lt;Name: <b>copy</b> + drop + store, Value: store&gt;(
    // we <b>use</b> &<b>mut</b> UID in several spots <b>for</b> access control
    object: &<b>mut</b> UID,
    name: Name,
    value: Value,
) {
    <b>let</b> object_addr = object.to_address();
    <b>let</b> hash = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_hash_type_and_key">hash_type_and_key</a>(object_addr, name);
    <b>assert</b>!(!<a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_has_child_object">has_child_object</a>(object_addr, hash), <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldAlreadyExists">EFieldAlreadyExists</a>);
    <b>let</b> field = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_Field">Field</a> {
        id: object::new_uid_from_hash(hash),
        name,
        value,
    };
    <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_add_child_object">add_child_object</a>(object_addr, field)
}
</code></pre>



</details>

<a name="stylus_dynamic_field_borrow"></a>

## Function `borrow`

Immutably borrows the <code>object</code>s dynamic field with the name specified by <code>name: Name</code>.
Aborts with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldDoesNotExist">EFieldDoesNotExist</a></code> if the object does not have a field with that name.
Aborts with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldTypeMismatch">EFieldTypeMismatch</a></code> if the field exists, but the value does not have the specified
type.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow">borrow</a>&lt;Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, name: Name): &Value
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow">borrow</a>&lt;Name: <b>copy</b> + drop + store, Value: store&gt;(object: &UID, name: Name): &Value {
    <b>let</b> object_addr = object.to_address();
    <b>let</b> hash = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_hash_type_and_key">hash_type_and_key</a>(object_addr, name);
    <b>let</b> field = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow_child_object">borrow_child_object</a>&lt;<a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_Field">Field</a>&lt;Name, Value&gt;&gt;(object, hash);
    &field.value
}
</code></pre>



</details>

<a name="stylus_dynamic_field_borrow_mut"></a>

## Function `borrow_mut`

Mutably borrows the <code>object</code>s dynamic field with the name specified by <code>name: Name</code>.
Aborts with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldDoesNotExist">EFieldDoesNotExist</a></code> if the object does not have a field with that name.
Aborts with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldTypeMismatch">EFieldTypeMismatch</a></code> if the field exists, but the value does not have the specified
type.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow_mut">borrow_mut</a>&lt;Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, name: Name): &<b>mut</b> Value
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow_mut">borrow_mut</a>&lt;Name: <b>copy</b> + drop + store, Value: store&gt;(
    object: &<b>mut</b> UID,
    name: Name,
): &<b>mut</b> Value {
    <b>let</b> object_addr = object.to_address();
    <b>let</b> hash = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_hash_type_and_key">hash_type_and_key</a>(object_addr, name);
    <b>let</b> field = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow_child_object_mut">borrow_child_object_mut</a>&lt;<a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_Field">Field</a>&lt;Name, Value&gt;&gt;(object, hash);
    &<b>mut</b> field.value
}
</code></pre>



</details>

<a name="stylus_dynamic_field_remove"></a>

## Function `remove`

Removes the <code>object</code>s dynamic field with the name specified by <code>name: Name</code> and returns the
bound value.
Aborts with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldDoesNotExist">EFieldDoesNotExist</a></code> if the object does not have a field with that name.
Aborts with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldTypeMismatch">EFieldTypeMismatch</a></code> if the field exists, but the value does not have the specified
type.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_remove">remove</a>&lt;Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, name: Name): Value
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_remove">remove</a>&lt;Name: <b>copy</b> + drop + store, Value: store&gt;(object: &<b>mut</b> UID, name: Name): Value {
    <b>let</b> object_addr = object.to_address();
    <b>let</b> hash = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_hash_type_and_key">hash_type_and_key</a>(object_addr, name);
    <b>let</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_Field">Field</a> { id, name: _, value } = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_remove_child_object">remove_child_object</a>&lt;<a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_Field">Field</a>&lt;Name, Value&gt;&gt;(object, hash);
    id.delete();
    value
}
</code></pre>



</details>

<a name="stylus_dynamic_field_exists_"></a>

## Function `exists_`

Returns true if and only if the <code>object</code> has a dynamic field with the name specified by
<code>name: Name</code> but without specifying the <code>Value</code> type


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_exists_">exists_</a>&lt;Name: <b>copy</b>, drop, store&gt;(object: &<a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, name: Name): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_exists_">exists_</a>&lt;Name: <b>copy</b> + drop + store&gt;(object: &UID, name: Name): bool {
    <b>let</b> object_addr = object.to_address();
    <b>let</b> hash = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_hash_type_and_key">hash_type_and_key</a>(object_addr, name);
    <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_has_child_object">has_child_object</a>(object_addr, hash)
}
</code></pre>



</details>

<a name="stylus_dynamic_field_remove_if_exists"></a>

## Function `remove_if_exists`

Removes the dynamic field if it exists. Returns the <code>some(Value)</code> if it exists or none otherwise.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_remove_if_exists">remove_if_exists</a>&lt;Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, name: Name): <a href="../../dependencies/std/option.md#std_option_Option">std::option::Option</a>&lt;Value&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_remove_if_exists">remove_if_exists</a>&lt;Name: <b>copy</b> + drop + store, Value: store&gt;(
    object: &<b>mut</b> UID,
    name: Name,
): Option&lt;Value&gt; {
    <b>if</b> (<a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_exists_">exists_</a>&lt;Name&gt;(object, name)) {
        option::some(<a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_remove">remove</a>(object, name))
    } <b>else</b> {
        option::none()
    }
}
</code></pre>



</details>

<a name="stylus_dynamic_field_exists_with_type"></a>

## Function `exists_with_type`

Returns true if and only if the <code>object</code> has a dynamic field with the name specified by
<code>name: Name</code> with an assigned value of type <code>Value</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_exists_with_type">exists_with_type</a>&lt;Name: <b>copy</b>, drop, store, Value: store&gt;(object: &<a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, name: Name): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_exists_with_type">exists_with_type</a>&lt;Name: <b>copy</b> + drop + store, Value: store&gt;(
    object: &UID,
    name: Name,
): bool {
    <b>let</b> object_addr = object.to_address();
    <b>let</b> hash = <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_hash_type_and_key">hash_type_and_key</a>(object_addr, name);
    // <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_has_child_object_with_ty">has_child_object_with_ty</a>&lt;<a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_Field">Field</a>&lt;Name, Value&gt;&gt;(object_addr, hash)
    // TODO: implement function above when typehash implemented
    <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_has_child_object">has_child_object</a>(object_addr, hash)
}
</code></pre>



</details>

<a name="stylus_dynamic_field_hash_type_and_key"></a>

## Function `hash_type_and_key`

May abort with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EBCSSerializationFailure">EBCSSerializationFailure</a></code>.


<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_hash_type_and_key">hash_type_and_key</a>&lt;K: <b>copy</b>, drop, store&gt;(parent: <b>address</b>, k: K): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_hash_type_and_key">hash_type_and_key</a>&lt;K: <b>copy</b> + drop + store&gt;(
    parent: <b>address</b>,
    k: K,
): <b>address</b>;
</code></pre>



</details>

<a name="stylus_dynamic_field_add_child_object"></a>

## Function `add_child_object`



<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_add_child_object">add_child_object</a>&lt;Child: key&gt;(parent: <b>address</b>, child: Child)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_add_child_object">add_child_object</a>&lt;Child: key&gt;(parent: <b>address</b>, child: Child);
</code></pre>



</details>

<a name="stylus_dynamic_field_borrow_child_object"></a>

## Function `borrow_child_object`

throws <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldDoesNotExist">EFieldDoesNotExist</a></code> if a child does not exist with that ID
or throws <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldTypeMismatch">EFieldTypeMismatch</a></code> if the type does not match,
and may also abort with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EBCSSerializationFailure">EBCSSerializationFailure</a></code>
we need two versions to return a reference or a mutable reference


<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow_child_object">borrow_child_object</a>&lt;Child: key&gt;(object: &<a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, id: <b>address</b>): &Child
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow_child_object">borrow_child_object</a>&lt;Child: key&gt;(object: &UID, id: <b>address</b>): &Child;
</code></pre>



</details>

<a name="stylus_dynamic_field_borrow_child_object_mut"></a>

## Function `borrow_child_object_mut`



<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow_child_object_mut">borrow_child_object_mut</a>&lt;Child: key&gt;(object: &<b>mut</b> <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, id: <b>address</b>): &<b>mut</b> Child
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_borrow_child_object_mut">borrow_child_object_mut</a>&lt;Child: key&gt;(
    object: &<b>mut</b> UID,
    id: <b>address</b>,
): &<b>mut</b> Child;
</code></pre>



</details>

<a name="stylus_dynamic_field_remove_child_object"></a>

## Function `remove_child_object`

throws <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldDoesNotExist">EFieldDoesNotExist</a></code> if a child does not exist with that ID
or throws <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EFieldTypeMismatch">EFieldTypeMismatch</a></code> if the type does not match,
and may also abort with <code><a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_EBCSSerializationFailure">EBCSSerializationFailure</a></code>.


<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_remove_child_object">remove_child_object</a>&lt;Child: key&gt;(object: &<a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a>, id: <b>address</b>): Child
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_remove_child_object">remove_child_object</a>&lt;Child: key&gt;(object: &UID, id: <b>address</b>): Child;
</code></pre>



</details>

<a name="stylus_dynamic_field_has_child_object"></a>

## Function `has_child_object`



<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_has_child_object">has_child_object</a>(parent: <b>address</b>, id: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_has_child_object">has_child_object</a>(parent: <b>address</b>, id: <b>address</b>): bool;
</code></pre>



</details>

<a name="stylus_dynamic_field_has_child_object_with_ty"></a>

## Function `has_child_object_with_ty`



<pre><code><b>public</b>(package) <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_has_child_object_with_ty">has_child_object_with_ty</a>&lt;Child: key&gt;(parent: <b>address</b>, id: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(package) <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field_has_child_object_with_ty">has_child_object_with_ty</a>&lt;Child: key&gt;(parent: <b>address</b>, id: <b>address</b>): bool;
</code></pre>



</details>
