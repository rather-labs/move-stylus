
<a name="hello_world_stack"></a>

# Module `hello_world::stack`



-  [Struct `Stack`](#hello_world_stack_Stack)
-  [Function `new`](#hello_world_stack_new)
-  [Function `push_back`](#hello_world_stack_push_back)
-  [Function `pop_back`](#hello_world_stack_pop_back)
-  [Function `size`](#hello_world_stack_size)


<pre><code><b>use</b> <a href="../dependencies/std/option.md#std_option">std::option</a>;
<b>use</b> <a href="../dependencies/std/vector.md#std_vector">std::vector</a>;
</code></pre>



<a name="hello_world_stack_Stack"></a>

## Struct `Stack`

Very simple stack implementation using the wrapper type pattern. Does not allow
accessing the elements unless they are popped.


<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/stack.md#hello_world_stack_Stack">Stack</a>&lt;T&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>0: vector&lt;T&gt;</code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_stack_new"></a>

## Function `new`

Create a new instance by wrapping the value.


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/stack.md#hello_world_stack_new">new</a>&lt;T&gt;(value: vector&lt;T&gt;): <a href="../hello_world/stack.md#hello_world_stack_Stack">hello_world::stack::Stack</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/stack.md#hello_world_stack_new">new</a>&lt;T&gt;(value: vector&lt;T&gt;): <a href="../hello_world/stack.md#hello_world_stack_Stack">Stack</a>&lt;T&gt; {
    <a href="../hello_world/stack.md#hello_world_stack_Stack">Stack</a>(value)
}
</code></pre>



</details>

<a name="hello_world_stack_push_back"></a>

## Function `push_back`

Push an element to the stack.


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/stack.md#hello_world_stack_push_back">push_back</a>&lt;T&gt;(v: &<b>mut</b> <a href="../hello_world/stack.md#hello_world_stack_Stack">hello_world::stack::Stack</a>&lt;T&gt;, el: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/stack.md#hello_world_stack_push_back">push_back</a>&lt;T&gt;(v: &<b>mut</b> <a href="../hello_world/stack.md#hello_world_stack_Stack">Stack</a>&lt;T&gt;, el: T) {
    v.0.<a href="../hello_world/stack.md#hello_world_stack_push_back">push_back</a>(el);
}
</code></pre>



</details>

<a name="hello_world_stack_pop_back"></a>

## Function `pop_back`

Pop an element from the stack. Unlike <code>vector</code>, this function won't
fail if the stack is empty and will return <code>None</code> instead.


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/stack.md#hello_world_stack_pop_back">pop_back</a>&lt;T&gt;(v: &<b>mut</b> <a href="../hello_world/stack.md#hello_world_stack_Stack">hello_world::stack::Stack</a>&lt;T&gt;): <a href="../dependencies/std/option.md#std_option_Option">std::option::Option</a>&lt;T&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/stack.md#hello_world_stack_pop_back">pop_back</a>&lt;T&gt;(v: &<b>mut</b> <a href="../hello_world/stack.md#hello_world_stack_Stack">Stack</a>&lt;T&gt;): Option&lt;T&gt; {
    <b>if</b> (v.0.length() == 0) option::none()
    <b>else</b> option::some(v.0.<a href="../hello_world/stack.md#hello_world_stack_pop_back">pop_back</a>())
}
</code></pre>



</details>

<a name="hello_world_stack_size"></a>

## Function `size`

Get the size of the stack.


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/stack.md#hello_world_stack_size">size</a>&lt;T&gt;(v: &<a href="../hello_world/stack.md#hello_world_stack_Stack">hello_world::stack::Stack</a>&lt;T&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../hello_world/stack.md#hello_world_stack_size">size</a>&lt;T&gt;(v: &<a href="../hello_world/stack.md#hello_world_stack_Stack">Stack</a>&lt;T&gt;): u64 {
    v.0.length()
}
</code></pre>



</details>
