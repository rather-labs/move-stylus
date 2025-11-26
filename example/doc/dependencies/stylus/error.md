
<a name="stylus_error"></a>

# Module `stylus::error`



-  [Function `revert`](#stylus_error_revert)


<pre><code></code></pre>



<a name="stylus_error_revert"></a>

## Function `revert`

Reverts the current transaction.

This function reverts the current transaction with a given error.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/error.md#stylus_error_revert">revert</a>&lt;T: <b>copy</b>, drop&gt;(error: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/error.md#stylus_error_revert">revert</a>&lt;T: <b>copy</b> + drop&gt;(error: T);
</code></pre>



</details>
