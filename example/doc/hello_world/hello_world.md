
<a name="hello_world_hello_world"></a>

# Module `hello_world::hello_world`



-  [Struct `Bar`](#hello_world_hello_world_Bar)
-  [Struct `Foo`](#hello_world_hello_world_Foo)
-  [Struct `Baz`](#hello_world_hello_world_Baz)
-  [Enum `TestEnum`](#hello_world_hello_world_TestEnum)
-  [Constants](#@Constants_0)
-  [Function `get_constant`](#hello_world_hello_world_get_constant)
-  [Function `get_constant_local`](#hello_world_hello_world_get_constant_local)
-  [Function `get_local`](#hello_world_hello_world_get_local)
-  [Function `get_copied_local`](#hello_world_hello_world_get_copied_local)
-  [Function `echo`](#hello_world_hello_world_echo)
-  [Function `echo_2`](#hello_world_hello_world_echo_2)
-  [Function `identity`](#hello_world_hello_world_identity)
-  [Function `identity_2`](#hello_world_hello_world_identity_2)
-  [Function `tx_context_properties`](#hello_world_hello_world_tx_context_properties)
-  [Function `fibonacci`](#hello_world_hello_world_fibonacci)
-  [Function `sum_special`](#hello_world_hello_world_sum_special)
-  [Function `create_foo_u16`](#hello_world_hello_world_create_foo_u16)
-  [Function `create_foo2_u16`](#hello_world_hello_world_create_foo2_u16)
-  [Function `create_baz_u16`](#hello_world_hello_world_create_baz_u16)
-  [Function `create_baz2_u16`](#hello_world_hello_world_create_baz2_u16)
-  [Function `multi_values_1`](#hello_world_hello_world_multi_values_1)
-  [Function `multi_values_2`](#hello_world_hello_world_multi_values_2)
-  [Function `echo_variant`](#hello_world_hello_world_echo_variant)
-  [Function `test_values`](#hello_world_hello_world_test_values)


<pre><code><b>use</b> <a href="../hello_world/another_mod.md#hello_world_another_mod">hello_world::another_mod</a>;
<b>use</b> <a href="../hello_world/other_mod.md#hello_world_other_mod">hello_world::other_mod</a>;
<b>use</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="hello_world_hello_world_Bar"></a>

## Struct `Bar`

Struct with generic type T


<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_Bar">Bar</a> <b>has</b> <b>copy</b>, drop
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
<code>b: u128</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_hello_world_Foo"></a>

## Struct `Foo`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_Foo">Foo</a>&lt;T&gt; <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>c: T</code>
</dt>
<dd>
</dd>
<dt>
<code>d: <a href="../hello_world/hello_world.md#hello_world_hello_world_Bar">hello_world::hello_world::Bar</a></code>
</dt>
<dd>
</dd>
<dt>
<code>e: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>f: bool</code>
</dt>
<dd>
</dd>
<dt>
<code>g: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>h: u256</code>
</dt>
<dd>
</dd>
<dt>
<code>i: vector&lt;u32&gt;</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_hello_world_Baz"></a>

## Struct `Baz`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_Baz">Baz</a>&lt;T&gt; <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>c: T</code>
</dt>
<dd>
</dd>
<dt>
<code>d: <a href="../hello_world/hello_world.md#hello_world_hello_world_Bar">hello_world::hello_world::Bar</a></code>
</dt>
<dd>
</dd>
<dt>
<code>e: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>f: bool</code>
</dt>
<dd>
</dd>
<dt>
<code>g: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>h: u256</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_hello_world_TestEnum"></a>

## Enum `TestEnum`



<pre><code><b>public</b> <b>enum</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_TestEnum">TestEnum</a> <b>has</b> drop
</code></pre>



<details>
<summary>Variants</summary>


<dl>
<dt>
Variant <code>FirstVariant</code>
</dt>
<dd>
</dd>
<dt>
Variant <code>SecondVariant</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="hello_world_hello_world_INT_AS_CONST"></a>



<pre><code><b>const</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_INT_AS_CONST">INT_AS_CONST</a>: u128 = 128128128;
</code></pre>



<a name="hello_world_hello_world_get_constant"></a>

## Function `get_constant`

Return a constant


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_get_constant">get_constant</a>(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_get_constant">get_constant</a>(): u128 {
  <a href="../hello_world/hello_world.md#hello_world_hello_world_INT_AS_CONST">INT_AS_CONST</a>
}
</code></pre>



</details>

<a name="hello_world_hello_world_get_constant_local"></a>

## Function `get_constant_local`

Set constant as local


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_get_constant_local">get_constant_local</a>(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_get_constant_local">get_constant_local</a>(): u128 {
  <b>let</b> x: u128 = <a href="../hello_world/hello_world.md#hello_world_hello_world_INT_AS_CONST">INT_AS_CONST</a>;
  x
}
</code></pre>



</details>

<a name="hello_world_hello_world_get_local"></a>

## Function `get_local`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_get_local">get_local</a>(_z: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_get_local">get_local</a>(_z: u128): u128 {
  <b>let</b> x: u128 = 100;
  <b>let</b> y: u128 = 50;
  <a href="../hello_world/hello_world.md#hello_world_hello_world_identity">identity</a>(x);
  <a href="../hello_world/hello_world.md#hello_world_hello_world_identity_2">identity_2</a>(x, y)
}
</code></pre>



</details>

<a name="hello_world_hello_world_get_copied_local"></a>

## Function `get_copied_local`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_get_copied_local">get_copied_local</a>(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_get_copied_local">get_copied_local</a>(): u128 {
  <b>let</b> x: u128 = 100;
  <b>let</b> y = x; // <b>copy</b>
  <b>let</b> <b>mut</b> _z = x; // <b>move</b>
  <a href="../hello_world/hello_world.md#hello_world_hello_world_identity">identity</a>(y);
  <a href="../hello_world/hello_world.md#hello_world_hello_world_identity">identity</a>(_z);
  _z = 111;
  y
}
</code></pre>



</details>

<a name="hello_world_hello_world_echo"></a>

## Function `echo`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_echo">echo</a>(x: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_echo">echo</a>(x: u128): u128 {
  <a href="../hello_world/hello_world.md#hello_world_hello_world_identity">identity</a>(x)
}
</code></pre>



</details>

<a name="hello_world_hello_world_echo_2"></a>

## Function `echo_2`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_echo_2">echo_2</a>(x: u128, y: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_echo_2">echo_2</a>(x: u128, y: u128): u128 {
  <a href="../hello_world/hello_world.md#hello_world_hello_world_identity_2">identity_2</a>(x, y)
}
</code></pre>



</details>

<a name="hello_world_hello_world_identity"></a>

## Function `identity`



<pre><code><b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_identity">identity</a>(x: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_identity">identity</a>(x: u128): u128 {
  x
}
</code></pre>



</details>

<a name="hello_world_hello_world_identity_2"></a>

## Function `identity_2`



<pre><code><b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_identity_2">identity_2</a>(_x: u128, y: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_identity_2">identity_2</a>(_x: u128, y: u128): u128 {
  y
}
</code></pre>



</details>

<a name="hello_world_hello_world_tx_context_properties"></a>

## Function `tx_context_properties`

Exposition of EVM global variables through TxContext object


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_tx_context_properties">tx_context_properties</a>(ctx: &<a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>): (<b>address</b>, u256, u64, u256, u64, u64, u64, u256)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_tx_context_properties">tx_context_properties</a>(ctx: &TxContext): (<b>address</b>, u256, u64, u256, u64, u64, u64, u256) {
    (
        ctx.sender(),
        ctx.msg_value(),
        ctx.block_number(),
        ctx.block_basefee(),
        ctx.block_gas_limit(),
        ctx.block_timestamp(),
        ctx.chain_id(),
        ctx.gas_price(),
    )
}
</code></pre>



</details>

<a name="hello_world_hello_world_fibonacci"></a>

## Function `fibonacci`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_fibonacci">fibonacci</a>(n: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_fibonacci">fibonacci</a>(n: u64): u64 {
    <b>if</b> (n == 0) <b>return</b> 0;
    <b>if</b> (n == 1) <b>return</b> 1;
    <b>let</b> <b>mut</b> a = 0;
    <b>let</b> <b>mut</b> b = 1;
    <b>let</b> <b>mut</b> count = 2;
    <b>while</b> (count &lt;= n) {
        <b>let</b> temp = a + b;
        a = b;
        b = temp;
        count = count + 1;
    };
    b
}
</code></pre>



</details>

<a name="hello_world_hello_world_sum_special"></a>

## Function `sum_special`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_sum_special">sum_special</a>(n: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_sum_special">sum_special</a>(n: u64): u64 {
    <b>let</b> <b>mut</b> total = 0;
    <b>let</b> <b>mut</b> i = 1;
    'outer: <b>loop</b> {
        <b>if</b> (i &gt; n) {
            <b>break</b>
        };
        <b>if</b> (i &gt; 1) {
            <b>let</b> <b>mut</b> j = 2;
            <b>let</b> <b>mut</b> x = 1;
            <b>while</b> (j * j &lt;= i) {
                <b>if</b> (i % j == 0) {
                    x = 0;
                    <b>break</b>
                };
                j = j + 1;
            };
            <b>if</b> (x == 1) {
                total = total + 7;
            };
        };
        i = i + 1;
    };
    total
}
</code></pre>



</details>

<a name="hello_world_hello_world_create_foo_u16"></a>

## Function `create_foo_u16`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_create_foo_u16">create_foo_u16</a>(a: u16, b: u16): <a href="../hello_world/hello_world.md#hello_world_hello_world_Foo">hello_world::hello_world::Foo</a>&lt;u16&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_create_foo_u16">create_foo_u16</a>(a: u16, b: u16): <a href="../hello_world/hello_world.md#hello_world_hello_world_Foo">Foo</a>&lt;u16&gt; {
    <b>let</b> <b>mut</b> foo = <a href="../hello_world/hello_world.md#hello_world_hello_world_Foo">Foo</a> {
        c: a,
        d: <a href="../hello_world/hello_world.md#hello_world_hello_world_Bar">Bar</a> { a: 42, b: 4242 },
        e: @0x7357,
        f: <b>true</b>,
        g: 1,
        h: 2,
        i: vector[0xFFFFFFFF],
    };
    foo.c = b;
    foo
}
</code></pre>



</details>

<a name="hello_world_hello_world_create_foo2_u16"></a>

## Function `create_foo2_u16`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_create_foo2_u16">create_foo2_u16</a>(a: u16, b: u16): (<a href="../hello_world/hello_world.md#hello_world_hello_world_Foo">hello_world::hello_world::Foo</a>&lt;u16&gt;, <a href="../hello_world/hello_world.md#hello_world_hello_world_Foo">hello_world::hello_world::Foo</a>&lt;u16&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_create_foo2_u16">create_foo2_u16</a>(a: u16, b: u16): (<a href="../hello_world/hello_world.md#hello_world_hello_world_Foo">Foo</a>&lt;u16&gt;, <a href="../hello_world/hello_world.md#hello_world_hello_world_Foo">Foo</a>&lt;u16&gt;) {
    <b>let</b> <b>mut</b> foo = <a href="../hello_world/hello_world.md#hello_world_hello_world_Foo">Foo</a> {
        c: a,
        d: <a href="../hello_world/hello_world.md#hello_world_hello_world_Bar">Bar</a> { a: 42, b: 4242 },
        e: @0x7357,
        f: <b>true</b>,
        g: 1,
        h: 2,
        i: vector[0xFFFFFFFF],
    };
    foo.c = b;
    (foo, <b>copy</b>(foo))
}
</code></pre>



</details>

<a name="hello_world_hello_world_create_baz_u16"></a>

## Function `create_baz_u16`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_create_baz_u16">create_baz_u16</a>(a: u16, _b: u16): <a href="../hello_world/hello_world.md#hello_world_hello_world_Baz">hello_world::hello_world::Baz</a>&lt;u16&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_create_baz_u16">create_baz_u16</a>(a: u16, _b: u16): <a href="../hello_world/hello_world.md#hello_world_hello_world_Baz">Baz</a>&lt;u16&gt; {
    <b>let</b> baz = <a href="../hello_world/hello_world.md#hello_world_hello_world_Baz">Baz</a> {
        c: a,
        d: <a href="../hello_world/hello_world.md#hello_world_hello_world_Bar">Bar</a> { a: 42, b: 4242 },
        e: @0x7357,
        f: <b>true</b>,
        g: 1,
        h: 2,
    };
    baz
}
</code></pre>



</details>

<a name="hello_world_hello_world_create_baz2_u16"></a>

## Function `create_baz2_u16`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_create_baz2_u16">create_baz2_u16</a>(a: u16, _b: u16): (<a href="../hello_world/hello_world.md#hello_world_hello_world_Baz">hello_world::hello_world::Baz</a>&lt;u16&gt;, <a href="../hello_world/hello_world.md#hello_world_hello_world_Baz">hello_world::hello_world::Baz</a>&lt;u16&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_create_baz2_u16">create_baz2_u16</a>(a: u16, _b: u16): (<a href="../hello_world/hello_world.md#hello_world_hello_world_Baz">Baz</a>&lt;u16&gt;, <a href="../hello_world/hello_world.md#hello_world_hello_world_Baz">Baz</a>&lt;u16&gt;) {
    <b>let</b> baz = <a href="../hello_world/hello_world.md#hello_world_hello_world_Baz">Baz</a> {
        c: a,
        d: <a href="../hello_world/hello_world.md#hello_world_hello_world_Bar">Bar</a> { a: 42, b: 4242 },
        e: @0x7357,
        f: <b>true</b>,
        g: 1,
        h: 2,
    };
    (baz, <b>copy</b>(baz))
}
</code></pre>



</details>

<a name="hello_world_hello_world_multi_values_1"></a>

## Function `multi_values_1`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_multi_values_1">multi_values_1</a>(): (vector&lt;u32&gt;, vector&lt;u128&gt;, bool, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_multi_values_1">multi_values_1</a>(): (vector&lt;u32&gt;, vector&lt;u128&gt;, bool, u64) {
    (vector[0xFFFFFFFF, 0xFFFFFFFF], vector[0xFFFFFFFFFF_u128], <b>true</b>, 42)
}
</code></pre>



</details>

<a name="hello_world_hello_world_multi_values_2"></a>

## Function `multi_values_2`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_multi_values_2">multi_values_2</a>(): (u8, bool, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_multi_values_2">multi_values_2</a>(): (u8, bool, u64) {
    (84, <b>true</b>, 42)
}
</code></pre>



</details>

<a name="hello_world_hello_world_echo_variant"></a>

## Function `echo_variant`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_echo_variant">echo_variant</a>(x: <a href="../hello_world/hello_world.md#hello_world_hello_world_TestEnum">hello_world::hello_world::TestEnum</a>): <a href="../hello_world/hello_world.md#hello_world_hello_world_TestEnum">hello_world::hello_world::TestEnum</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_echo_variant">echo_variant</a>(x: <a href="../hello_world/hello_world.md#hello_world_hello_world_TestEnum">TestEnum</a>): <a href="../hello_world/hello_world.md#hello_world_hello_world_TestEnum">TestEnum</a> {
    x
}
</code></pre>



</details>

<a name="hello_world_hello_world_test_values"></a>

## Function `test_values`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_test_values">test_values</a>(test: &<a href="../hello_world/other_mod.md#hello_world_other_mod_Test">hello_world::other_mod::Test</a>): (u8, u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/hello_world.md#hello_world_hello_world_test_values">test_values</a>(test: &Test): (u8, u8) {
    test.get_test_values()
}
</code></pre>



</details>
