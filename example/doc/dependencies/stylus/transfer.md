
<a name="stylus_transfer"></a>

# Module `stylus::transfer`



-  [Function `transfer`](#stylus_transfer_transfer)
-  [Function `freeze_object`](#stylus_transfer_freeze_object)
-  [Function `share_object`](#stylus_transfer_share_object)


<pre><code></code></pre>



<a name="stylus_transfer_transfer"></a>

## Function `transfer`

Transfer ownership of <code>obj</code> to <code>recipient</code>. <code>obj</code> must have the <code>key</code> attribute,
which (in turn) ensures that <code>obj</code> has a globally unique ID.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/transfer.md#stylus_transfer_transfer">transfer</a>&lt;T: key&gt;(obj: T, recipient: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/transfer.md#stylus_transfer_transfer">transfer</a>&lt;T: key&gt;(obj: T, recipient: <b>address</b>);
</code></pre>



</details>

<a name="stylus_transfer_freeze_object"></a>

## Function `freeze_object`

Freezes <code>obj</code>. After freezing <code>obj</code> becomes immutable and can no longer be transferred or
mutated.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/transfer.md#stylus_transfer_freeze_object">freeze_object</a>&lt;T: key&gt;(obj: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/transfer.md#stylus_transfer_freeze_object">freeze_object</a>&lt;T: key&gt;(obj: T);
</code></pre>



</details>

<a name="stylus_transfer_share_object"></a>

## Function `share_object`

Turns the given object into a mutable shared object that everyone can access and mutate.
This is irreversible, i.e. once an object is shared, it will stay shared forever.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/transfer.md#stylus_transfer_share_object">share_object</a>&lt;T: key&gt;(obj: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/transfer.md#stylus_transfer_share_object">share_object</a>&lt;T: key&gt;(obj: T);
</code></pre>



</details>
