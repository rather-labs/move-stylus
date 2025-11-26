
<a name="stylus_event"></a>

# Module `stylus::event`

Events module.

Defines the functions to publicly log data in the blockchain.

For more information:
https://docs.soliditylang.org/en/v0.8.19/abi-spec.html#events
https://docs.arbitrum.io/stylus-by-example/basic_examples/events


-  [Function `emit`](#stylus_event_emit)


<pre><code></code></pre>



<a name="stylus_event_emit"></a>

## Function `emit`

Emits an event in the topic 0.

This function It ensures that an event will be logged in a Solidity ABI-compatible format.


<pre><code><b>public</b> <b>fun</b> <a href="../../dependencies/stylus/event.md#stylus_event_emit">emit</a>&lt;T: <b>copy</b>, drop&gt;(event: T)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>native</b> <b>fun</b> <a href="../../dependencies/stylus/event.md#stylus_event_emit">emit</a>&lt;T: <b>copy</b> + drop&gt;(event: T);
</code></pre>



</details>
