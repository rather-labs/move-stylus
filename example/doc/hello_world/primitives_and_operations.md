
<a name="hello_world_primitives_and_operations"></a>

# Module `hello_world::primitives_and_operations`



-  [Constants](#@Constants_0)
-  [Function `cast_u8`](#hello_world_primitives_and_operations_cast_u8)
-  [Function `sum_u256`](#hello_world_primitives_and_operations_sum_u256)
-  [Function `sub_u128`](#hello_world_primitives_and_operations_sub_u128)
-  [Function `mul_u64`](#hello_world_primitives_and_operations_mul_u64)
-  [Function `div_u32`](#hello_world_primitives_and_operations_div_u32)
-  [Function `mod_u16`](#hello_world_primitives_and_operations_mod_u16)
-  [Function `or_u256`](#hello_world_primitives_and_operations_or_u256)
-  [Function `xor_u128`](#hello_world_primitives_and_operations_xor_u128)
-  [Function `and_u64`](#hello_world_primitives_and_operations_and_u64)
-  [Function `shift_left_u32`](#hello_world_primitives_and_operations_shift_left_u32)
-  [Function `shift_right_u16`](#hello_world_primitives_and_operations_shift_right_u16)
-  [Function `not_true`](#hello_world_primitives_and_operations_not_true)
-  [Function `not`](#hello_world_primitives_and_operations_not)
-  [Function `and`](#hello_world_primitives_and_operations_and)
-  [Function `or`](#hello_world_primitives_and_operations_or)
-  [Function `less_than_u256`](#hello_world_primitives_and_operations_less_than_u256)
-  [Function `less_than_eq_u128`](#hello_world_primitives_and_operations_less_than_eq_u128)
-  [Function `greater_than_u64`](#hello_world_primitives_and_operations_greater_than_u64)
-  [Function `greater_than_eq_u32`](#hello_world_primitives_and_operations_greater_than_eq_u32)
-  [Function `vec_from_u256`](#hello_world_primitives_and_operations_vec_from_u256)
-  [Function `vec_len_u128`](#hello_world_primitives_and_operations_vec_len_u128)
-  [Function `vec_pop_back_u64`](#hello_world_primitives_and_operations_vec_pop_back_u64)
-  [Function `vec_swap_u32`](#hello_world_primitives_and_operations_vec_swap_u32)
-  [Function `vec_push_back_u16`](#hello_world_primitives_and_operations_vec_push_back_u16)


<pre><code></code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="hello_world_primitives_and_operations_BOOL_AS_CONST"></a>



<pre><code><b>const</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_BOOL_AS_CONST">BOOL_AS_CONST</a>: bool = <b>true</b>;
</code></pre>



<a name="hello_world_primitives_and_operations_cast_u8"></a>

## Function `cast_u8`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_cast_u8">cast_u8</a>(x: u128): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_cast_u8">cast_u8</a>(x: u128): u8 {
    x <b>as</b> u8
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_sum_u256"></a>

## Function `sum_u256`

Arithmetic operations


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_sum_u256">sum_u256</a>(x: u256, y: u256): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_sum_u256">sum_u256</a>(x: u256, y: u256): u256 {
    x + y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_sub_u128"></a>

## Function `sub_u128`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_sub_u128">sub_u128</a>(x: u128, y: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_sub_u128">sub_u128</a>(x: u128, y: u128): u128 {
    x - y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_mul_u64"></a>

## Function `mul_u64`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_mul_u64">mul_u64</a>(x: u64, y: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_mul_u64">mul_u64</a>(x: u64, y: u64): u64 {
    x * y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_div_u32"></a>

## Function `div_u32`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_div_u32">div_u32</a>(x: u32, y: u32): u32
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_div_u32">div_u32</a>(x: u32, y: u32): u32 {
    x / y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_mod_u16"></a>

## Function `mod_u16`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_mod_u16">mod_u16</a>(x: u16, y: u16): u16
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_mod_u16">mod_u16</a>(x: u16, y: u16): u16 {
    x % y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_or_u256"></a>

## Function `or_u256`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_or_u256">or_u256</a>(x: u256, y: u256): u256
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_or_u256">or_u256</a>(x: u256, y: u256): u256 {
    x | y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_xor_u128"></a>

## Function `xor_u128`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_xor_u128">xor_u128</a>(x: u128, y: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_xor_u128">xor_u128</a>(x: u128, y: u128): u128 {
    x ^ y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_and_u64"></a>

## Function `and_u64`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_and_u64">and_u64</a>(x: u64, y: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_and_u64">and_u64</a>(x: u64, y: u64): u64 {
    x & y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_shift_left_u32"></a>

## Function `shift_left_u32`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_shift_left_u32">shift_left_u32</a>(x: u32, slots: u8): u32
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_shift_left_u32">shift_left_u32</a>(x: u32, slots: u8): u32 {
    x &lt;&lt; slots
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_shift_right_u16"></a>

## Function `shift_right_u16`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_shift_right_u16">shift_right_u16</a>(x: u16, slots: u8): u16
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_shift_right_u16">shift_right_u16</a>(x: u16, slots: u8): u16 {
    x &gt;&gt; slots
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_not_true"></a>

## Function `not_true`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_not_true">not_true</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_not_true">not_true</a>(): bool {
  !<a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_BOOL_AS_CONST">BOOL_AS_CONST</a>
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_not"></a>

## Function `not`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_not">not</a>(x: bool): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_not">not</a>(x: bool): bool {
  !x
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_and"></a>

## Function `and`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_and">and</a>(x: bool, y: bool): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_and">and</a>(x: bool, y: bool): bool {
  x && y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_or"></a>

## Function `or`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_or">or</a>(x: bool, y: bool): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_or">or</a>(x: bool, y: bool): bool {
  x || y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_less_than_u256"></a>

## Function `less_than_u256`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_less_than_u256">less_than_u256</a>(a: u256, b: u256): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_less_than_u256">less_than_u256</a>(a: u256, b: u256): bool {
    a &lt; b
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_less_than_eq_u128"></a>

## Function `less_than_eq_u128`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_less_than_eq_u128">less_than_eq_u128</a>(a: u128, b: u128): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_less_than_eq_u128">less_than_eq_u128</a>(a: u128, b: u128): bool {
    a &lt;= b
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_greater_than_u64"></a>

## Function `greater_than_u64`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_greater_than_u64">greater_than_u64</a>(a: u64, b: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_greater_than_u64">greater_than_u64</a>(a: u64, b: u64): bool {
    a &gt; b
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_greater_than_eq_u32"></a>

## Function `greater_than_eq_u32`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_greater_than_eq_u32">greater_than_eq_u32</a>(a: u32, b: u32): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_greater_than_eq_u32">greater_than_eq_u32</a>(a: u32, b: u32): bool {
    a &gt;= b
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_vec_from_u256"></a>

## Function `vec_from_u256`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_from_u256">vec_from_u256</a>(x: u256, y: u256): vector&lt;u256&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_from_u256">vec_from_u256</a>(x: u256, y: u256): vector&lt;u256&gt; {
  <b>let</b> z = vector[x, y, x];
  z
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_vec_len_u128"></a>

## Function `vec_len_u128`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_len_u128">vec_len_u128</a>(x: vector&lt;u128&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_len_u128">vec_len_u128</a>(x: vector&lt;u128&gt;): u64 {
  x.length()
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_vec_pop_back_u64"></a>

## Function `vec_pop_back_u64`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_pop_back_u64">vec_pop_back_u64</a>(x: vector&lt;u64&gt;): vector&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_pop_back_u64">vec_pop_back_u64</a>(x: vector&lt;u64&gt;): vector&lt;u64&gt; {
  <b>let</b> <b>mut</b> y = x;
  y.pop_back();
  y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_vec_swap_u32"></a>

## Function `vec_swap_u32`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_swap_u32">vec_swap_u32</a>(x: vector&lt;u32&gt;, id1: u64, id2: u64): vector&lt;u32&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_swap_u32">vec_swap_u32</a>(x: vector&lt;u32&gt;, id1: u64, id2: u64): vector&lt;u32&gt; {
  <b>let</b> <b>mut</b> y = x;
  y.swap(id1, id2);
  y
}
</code></pre>



</details>

<a name="hello_world_primitives_and_operations_vec_push_back_u16"></a>

## Function `vec_push_back_u16`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_push_back_u16">vec_push_back_u16</a>(x: vector&lt;u16&gt;, y: u16): vector&lt;u16&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/primitives_and_operations.md#hello_world_primitives_and_operations_vec_push_back_u16">vec_push_back_u16</a>(x: vector&lt;u16&gt;, y: u16): vector&lt;u16&gt; {
  <b>let</b> <b>mut</b> z = x;
  z.push_back(y);
  z
}
</code></pre>



</details>
