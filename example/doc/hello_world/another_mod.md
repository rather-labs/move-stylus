
<a name="hello_world_another_mod"></a>

# Module `hello_world::another_mod`



-  [Struct `AnotherTest`](#hello_world_another_mod_AnotherTest)
-  [Function `create_another_test`](#hello_world_another_mod_create_another_test)
-  [Function `get_another_test_value`](#hello_world_another_mod_get_another_test_value)
-  [Function `generic_identity_2`](#hello_world_another_mod_generic_identity_2)


<pre><code></code></pre>



<a name="hello_world_another_mod_AnotherTest"></a>

## Struct `AnotherTest`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/another_mod.md#hello_world_another_mod_AnotherTest">AnotherTest</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>0: u8</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_another_mod_create_another_test"></a>

## Function `create_another_test`



<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/another_mod.md#hello_world_another_mod_create_another_test">create_another_test</a>(x: u8): <a href="../hello_world/another_mod.md#hello_world_another_mod_AnotherTest">hello_world::another_mod::AnotherTest</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/another_mod.md#hello_world_another_mod_create_another_test">create_another_test</a>(x: u8): <a href="../hello_world/another_mod.md#hello_world_another_mod_AnotherTest">AnotherTest</a> {
    <a href="../hello_world/another_mod.md#hello_world_another_mod_AnotherTest">AnotherTest</a>(x)
}
</code></pre>



</details>

<a name="hello_world_another_mod_get_another_test_value"></a>

## Function `get_another_test_value`



<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/another_mod.md#hello_world_another_mod_get_another_test_value">get_another_test_value</a>(self: &<a href="../hello_world/another_mod.md#hello_world_another_mod_AnotherTest">hello_world::another_mod::AnotherTest</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>entry</b> <b>fun</b> <a href="../hello_world/another_mod.md#hello_world_another_mod_get_another_test_value">get_another_test_value</a>(self: &<a href="../hello_world/another_mod.md#hello_world_another_mod_AnotherTest">AnotherTest</a>): u8 {
    <b>let</b> <a href="../hello_world/another_mod.md#hello_world_another_mod_AnotherTest">AnotherTest</a>(value) = self;
    *value
}
</code></pre>



</details>

<a name="hello_world_another_mod_generic_identity_2"></a>

## Function `generic_identity_2`



<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/another_mod.md#hello_world_another_mod_generic_identity_2">generic_identity_2</a>&lt;T&gt;(t: T): T
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/another_mod.md#hello_world_another_mod_generic_identity_2">generic_identity_2</a>&lt;T&gt;(t: T): T {
    t
}
</code></pre>



</details>
