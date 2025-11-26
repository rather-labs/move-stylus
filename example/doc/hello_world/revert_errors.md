
<a name="hello_world_revert_errors"></a>

# Module `hello_world::revert_errors`



-  [Struct `Error_`](#hello_world_revert_errors_Error_)
-  [Struct `CustomError`](#hello_world_revert_errors_CustomError)
-  [Struct `CustomError2`](#hello_world_revert_errors_CustomError2)
-  [Struct `CustomError3`](#hello_world_revert_errors_CustomError3)
-  [Struct `NestedStruct`](#hello_world_revert_errors_NestedStruct)
-  [Struct `NestedStruct2`](#hello_world_revert_errors_NestedStruct2)
-  [Struct `CustomError4`](#hello_world_revert_errors_CustomError4)
-  [Enum `MyEnum`](#hello_world_revert_errors_MyEnum)
-  [Function `revert_standard_error`](#hello_world_revert_errors_revert_standard_error)
-  [Function `revert_custom_error`](#hello_world_revert_errors_revert_custom_error)
-  [Function `revert_custom_error2`](#hello_world_revert_errors_revert_custom_error2)
-  [Function `revert_custom_error3`](#hello_world_revert_errors_revert_custom_error3)
-  [Function `revert_custom_error4`](#hello_world_revert_errors_revert_custom_error4)


<pre><code><b>use</b> <a href="../dependencies/std/ascii.md#std_ascii">std::ascii</a>;
<b>use</b> <a href="../dependencies/std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../dependencies/std/vector.md#std_vector">std::vector</a>;
<b>use</b> <a href="../dependencies/stylus/error.md#stylus_error">stylus::error</a>;
</code></pre>



<a name="hello_world_revert_errors_Error_"></a>

## Struct `Error_`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_Error_">Error_</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>0: <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_revert_errors_CustomError"></a>

## Struct `CustomError`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_CustomError">CustomError</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>error_message: <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a></code>
</dt>
<dd>
</dd>
<dt>
<code>error_code: u64</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_revert_errors_CustomError2"></a>

## Struct `CustomError2`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_CustomError2">CustomError2</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>a: bool</code>
</dt>
<dd>
</dd>
<dt>
<code>b: u8</code>
</dt>
<dd>
</dd>
<dt>
<code>c: u16</code>
</dt>
<dd>
</dd>
<dt>
<code>d: u32</code>
</dt>
<dd>
</dd>
<dt>
<code>e: u64</code>
</dt>
<dd>
</dd>
<dt>
<code>f: u128</code>
</dt>
<dd>
</dd>
<dt>
<code>g: u256</code>
</dt>
<dd>
</dd>
<dt>
<code>h: <b>address</b></code>
</dt>
<dd>
</dd>
<dt>
<code>i: <a href="../hello_world/revert_errors.md#hello_world_revert_errors_MyEnum">hello_world::revert_errors::MyEnum</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_revert_errors_CustomError3"></a>

## Struct `CustomError3`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_CustomError3">CustomError3</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>a: vector&lt;u32&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>b: vector&lt;u128&gt;</code>
</dt>
<dd>
</dd>
<dt>
<code>c: vector&lt;vector&lt;u64&gt;&gt;</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_revert_errors_NestedStruct"></a>

## Struct `NestedStruct`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_NestedStruct">NestedStruct</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>0: <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_revert_errors_NestedStruct2"></a>

## Struct `NestedStruct2`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_NestedStruct2">NestedStruct2</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>a: <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a></code>
</dt>
<dd>
</dd>
<dt>
<code>b: u64</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_revert_errors_CustomError4"></a>

## Struct `CustomError4`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_CustomError4">CustomError4</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>a: <a href="../hello_world/revert_errors.md#hello_world_revert_errors_NestedStruct">hello_world::revert_errors::NestedStruct</a></code>
</dt>
<dd>
</dd>
<dt>
<code>b: <a href="../hello_world/revert_errors.md#hello_world_revert_errors_NestedStruct2">hello_world::revert_errors::NestedStruct2</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_revert_errors_MyEnum"></a>

## Enum `MyEnum`



<pre><code><b>public</b> <b>enum</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_MyEnum">MyEnum</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Variants</summary>


<dl>
<dt>
Variant <code>A</code>
</dt>
<dd>
</dd>
<dt>
Variant <code>B</code>
</dt>
<dd>
</dd>
<dt>
Variant <code>C</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_revert_errors_revert_standard_error"></a>

## Function `revert_standard_error`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_standard_error">revert_standard_error</a>(s: <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_standard_error">revert_standard_error</a>(s: String) {
    <b>let</b> error = <a href="../hello_world/revert_errors.md#hello_world_revert_errors_Error_">Error_</a>(s);
    revert(error);
}
</code></pre>



</details>

<a name="hello_world_revert_errors_revert_custom_error"></a>

## Function `revert_custom_error`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_custom_error">revert_custom_error</a>(s: <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a>, code: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_custom_error">revert_custom_error</a>(s: String, code: u64) {
    revert( <a href="../hello_world/revert_errors.md#hello_world_revert_errors_CustomError">CustomError</a> { error_message: s, error_code: code });
}
</code></pre>



</details>

<a name="hello_world_revert_errors_revert_custom_error2"></a>

## Function `revert_custom_error2`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_custom_error2">revert_custom_error2</a>(a: bool, b: u8, c: u16, d: u32, e: u64, f: u128, g: u256, h: <b>address</b>, i: <a href="../hello_world/revert_errors.md#hello_world_revert_errors_MyEnum">hello_world::revert_errors::MyEnum</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_custom_error2">revert_custom_error2</a>(a: bool, b: u8, c: u16, d: u32, e: u64, f: u128, g: u256, h: <b>address</b>, i: <a href="../hello_world/revert_errors.md#hello_world_revert_errors_MyEnum">MyEnum</a>) {
    revert(<a href="../hello_world/revert_errors.md#hello_world_revert_errors_CustomError2">CustomError2</a> { a, b, c, d, e, f, g, h, i });
}
</code></pre>



</details>

<a name="hello_world_revert_errors_revert_custom_error3"></a>

## Function `revert_custom_error3`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_custom_error3">revert_custom_error3</a>(a: vector&lt;u32&gt;, b: vector&lt;u128&gt;, c: vector&lt;vector&lt;u64&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_custom_error3">revert_custom_error3</a>(a: vector&lt;u32&gt;, b: vector&lt;u128&gt;, c: vector&lt;vector&lt;u64&gt;&gt;) {
    revert(<a href="../hello_world/revert_errors.md#hello_world_revert_errors_CustomError3">CustomError3</a> { a, b, c });
}
</code></pre>



</details>

<a name="hello_world_revert_errors_revert_custom_error4"></a>

## Function `revert_custom_error4`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_custom_error4">revert_custom_error4</a>(a: <a href="../dependencies/std/ascii.md#std_ascii_String">std::ascii::String</a>, b: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/revert_errors.md#hello_world_revert_errors_revert_custom_error4">revert_custom_error4</a>(a: String, b: u64) {
    <b>let</b> error = <a href="../hello_world/revert_errors.md#hello_world_revert_errors_NestedStruct">NestedStruct</a>(a);
    <b>let</b> custom_error = <a href="../hello_world/revert_errors.md#hello_world_revert_errors_NestedStruct2">NestedStruct2</a> { a, b };
    <b>let</b> error = <a href="../hello_world/revert_errors.md#hello_world_revert_errors_CustomError4">CustomError4</a> { a: error, b: custom_error };
    revert(error);
}
</code></pre>



</details>
