
<a name="hello_world_other_mod"></a>

# Module `hello_world::other_mod`



-  [Struct `Test`](#hello_world_other_mod_Test)
-  [Function `get_test_values`](#hello_world_other_mod_get_test_values)
-  [Function `generic_identity`](#hello_world_other_mod_generic_identity)
-  [Function `generic_identity_two_types`](#hello_world_other_mod_generic_identity_two_types)


<pre><code><b>use</b> <a href="../hello_world/another_mod.md#hello_world_another_mod">hello_world::another_mod</a>;
</code></pre>



<a name="hello_world_other_mod_Test"></a>

## Struct `Test`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/other_mod.md#hello_world_other_mod_Test">Test</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>0: u8</code>
</dt>
<dd>
</dd>
<dt>
<code>1: <a href="../hello_world/another_mod.md#hello_world_another_mod_AnotherTest">hello_world::another_mod::AnotherTest</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_other_mod_get_test_values"></a>

## Function `get_test_values`



<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/other_mod.md#hello_world_other_mod_get_test_values">get_test_values</a>(self: &<a href="../hello_world/other_mod.md#hello_world_other_mod_Test">hello_world::other_mod::Test</a>): (u8, u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/other_mod.md#hello_world_other_mod_get_test_values">get_test_values</a>(self: &<a href="../hello_world/other_mod.md#hello_world_other_mod_Test">Test</a>): (u8, u8) {
    <b>let</b> <a href="../hello_world/other_mod.md#hello_world_other_mod_Test">Test</a>(value, another_test) = self;
    (*value, another_test.get_another_test_value())
}
</code></pre>



</details>

<a name="hello_world_other_mod_generic_identity"></a>

## Function `generic_identity`



<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/other_mod.md#hello_world_other_mod_generic_identity">generic_identity</a>&lt;T&gt;(t: T): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/other_mod.md#hello_world_other_mod_generic_identity">generic_identity</a>&lt;T&gt;(t: T): T {
     generic_identity_2(t)
}
</code></pre>



</details>

<a name="hello_world_other_mod_generic_identity_two_types"></a>

## Function `generic_identity_two_types`



<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/other_mod.md#hello_world_other_mod_generic_identity_two_types">generic_identity_two_types</a>&lt;T, U&gt;(t: T, u: U): (T, U)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/other_mod.md#hello_world_other_mod_generic_identity_two_types">generic_identity_two_types</a>&lt;T, U&gt;(t: T, u: U): (T, U) {
    (
        <a href="../hello_world/other_mod.md#hello_world_other_mod_generic_identity">generic_identity</a>(t),
        <a href="../hello_world/other_mod.md#hello_world_other_mod_generic_identity">generic_identity</a>(u),
    )
}
</code></pre>



</details>
