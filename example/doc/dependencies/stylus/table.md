
<a name="stylus_table"></a>

# Module `stylus::table`

A table is a map-like collection. But unlike a traditional collection, it's keys and values are
not stored within the <code><a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a></code> value, but instead are stored using Sui's object system. The
<code><a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a></code> struct acts only as a handle into the object system to retrieve those keys and values.
Note that this means that <code><a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a></code> values with exactly the same key-value mapping will not be
equal, with <code>==</code>, at runtime. For example
```
let table1 = table::new<u64, bool>();
let table2 = table::new<u64, bool>();
table::add(&mut table1, 0, false);
table::add(&mut table1, 1, true);
table::add(&mut table2, 0, false);
table::add(&mut table2, 1, true);
// table1 does not equal table2, despite having the same entries
assert!(&table1 != &table2);
```


-  [Struct `Table`](#stylus_table_Table)
-  [Constants](#@Constants_0)
-  [Function `new`](#stylus_table_new)
-  [Function `add`](#stylus_table_add)
-  [Function `borrow`](#stylus_table_borrow)
-  [Function `borrow_mut`](#stylus_table_borrow_mut)
-  [Function `remove`](#stylus_table_remove)
-  [Function `contains`](#stylus_table_contains)
-  [Function `length`](#stylus_table_length)
-  [Function `is_empty`](#stylus_table_is_empty)
-  [Function `destroy_empty`](#stylus_table_destroy_empty)
-  [Function `drop`](#stylus_table_drop)


<pre><code><b>use</b> <a href="../../dependencies/std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../../dependencies/std/vector.md#std_vector">std::vector</a>;
<b>use</b> <a href="../../dependencies/stylus/dynamic_field.md#stylus_dynamic_field">stylus::dynamic_field</a>;
<b>use</b> <a href="../../dependencies/stylus/event.md#stylus_event">stylus::event</a>;
<b>use</b> <a href="../../dependencies/stylus/object.md#stylus_object">stylus::object</a>;
<b>use</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="stylus_table_Table"></a>

## Struct `Table`



<pre><code><b>public</b> <b>struct</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;<b>phantom</b> K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, <b>phantom</b> V: store&gt; <b>has</b> key, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a></code>
</dt>
<dd>
 the ID of this table
</dd>
<dt>
<code>size: u64</code>
</dt>
<dd>
 the number of key-value pairs in the table
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="stylus_table_ETableNotEmpty"></a>



<pre><code><b>const</b> <a href="../../dependencies/stylus/table.md#stylus_table_ETableNotEmpty">ETableNotEmpty</a>: u64 = 0;
</code></pre>



<a name="stylus_table_new"></a>

## Function `new`

Creates a new, empty table


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_new">new</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: store&gt;(ctx: &<b>mut</b> <a href="../../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): <a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_new">new</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: store&gt;(ctx: &<b>mut</b> TxContext): <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt; {
    <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a> {
        id: object::new(ctx),
        size: 0,
    }
}
</code></pre>



</details>

<a name="stylus_table_add"></a>

## Function `add`

Adds a key-value pair to the table <code>table: &<b>mut</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;</code>
Aborts with <code>sui::dynamic_field::EFieldAlreadyExists</code> if the table already has an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_add">add</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: store&gt;(table: &<b>mut</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;, k: K, v: V)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_add">add</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: store&gt;(table: &<b>mut</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;, k: K, v: V) {
    field::add(&<b>mut</b> table.id, k, v);
    table.size = table.size + 1;
}
</code></pre>



</details>

<a name="stylus_table_borrow"></a>

## Function `borrow`

Immutable borrows the value associated with the key in the table <code>table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;</code>.
Aborts with <code>sui::dynamic_field::EFieldDoesNotExist</code> if the table does not have an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_borrow">borrow</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: store&gt;(table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;, k: K): &V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_borrow">borrow</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: store&gt;(table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;, k: K): &V {
    field::borrow(&table.id, k)
}
</code></pre>



</details>

<a name="stylus_table_borrow_mut"></a>

## Function `borrow_mut`

Mutably borrows the value associated with the key in the table <code>table: &<b>mut</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;</code>.
Aborts with <code>sui::dynamic_field::EFieldDoesNotExist</code> if the table does not have an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: store&gt;(table: &<b>mut</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;, k: K): &<b>mut</b> V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_borrow_mut">borrow_mut</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: store&gt;(table: &<b>mut</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;, k: K): &<b>mut</b> V {
    field::borrow_mut(&<b>mut</b> table.id, k)
}
</code></pre>



</details>

<a name="stylus_table_remove"></a>

## Function `remove`

Removes the key-value pair in the table <code>table: &<b>mut</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;</code> and returns the value.
Aborts with <code>sui::dynamic_field::EFieldDoesNotExist</code> if the table does not have an entry with
that key <code>k: K</code>.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_remove">remove</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: store&gt;(table: &<b>mut</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;, k: K): V
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_remove">remove</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: store&gt;(table: &<b>mut</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;, k: K): V {
    <b>let</b> v = field::remove(&<b>mut</b> table.id, k);
    table.size = table.size - 1;
    v
}
</code></pre>



</details>

<a name="stylus_table_contains"></a>

## Function `contains`

Returns true if there is a value associated with the key <code>k: K</code> in table <code>table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;</code>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_contains">contains</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: store&gt;(table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;, k: K): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_contains">contains</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: store&gt;(table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;, k: K): bool {
    field::exists_with_type&lt;K, V&gt;(&table.id, k)
}
</code></pre>



</details>

<a name="stylus_table_length"></a>

## Function `length`

Returns the size of the table, the number of key-value pairs


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_length">length</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: store&gt;(table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_length">length</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: store&gt;(table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;): u64 {
    table.size
}
</code></pre>



</details>

<a name="stylus_table_is_empty"></a>

## Function `is_empty`

Returns true if the table is empty (if <code><a href="../../dependencies/stylus/table.md#stylus_table_length">length</a></code> returns <code>0</code>)


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_is_empty">is_empty</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: store&gt;(table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_is_empty">is_empty</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: store&gt;(table: &<a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;): bool {
    table.size == 0
}
</code></pre>



</details>

<a name="stylus_table_destroy_empty"></a>

## Function `destroy_empty`

Destroys an empty table
Aborts with <code><a href="../../dependencies/stylus/table.md#stylus_table_ETableNotEmpty">ETableNotEmpty</a></code> if the table still contains values


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_destroy_empty">destroy_empty</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: store&gt;(table: <a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_destroy_empty">destroy_empty</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: store&gt;(table: <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;) {
    <b>let</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a> { id, size } = table;
    <b>assert</b>!(size == 0, <a href="../../dependencies/stylus/table.md#stylus_table_ETableNotEmpty">ETableNotEmpty</a>);
    id.delete()
}
</code></pre>



</details>

<a name="stylus_table_drop"></a>

## Function `drop`

Drop a possibly non-empty table.
Usable only if the value type <code>V</code> has the <code><a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a></code> ability


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>&lt;K: <b>copy</b>, <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store, V: <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>, store&gt;(table: <a href="../../dependencies/stylus/table.md#stylus_table_Table">stylus::table::Table</a>&lt;K, V&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a>&lt;K: <b>copy</b> + <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store, V: <a href="../../dependencies/stylus/table.md#stylus_table_drop">drop</a> + store&gt;(table: <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a>&lt;K, V&gt;) {
    <b>let</b> <a href="../../dependencies/stylus/table.md#stylus_table_Table">Table</a> { id, size: _ } = table;
    id.delete()
}
</code></pre>



</details>
