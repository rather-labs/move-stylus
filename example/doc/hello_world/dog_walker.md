
<a name="hello_world_dog_walker"></a>

# Module `hello_world::dog_walker`



-  [Struct `IWalkTheDog`](#hello_world_dog_walker_IWalkTheDog)
-  [Struct `CanWalkDogCap`](#hello_world_dog_walker_CanWalkDogCap)
-  [Function `create`](#hello_world_dog_walker_create)
-  [Function `walk_the_dog`](#hello_world_dog_walker_walk_the_dog)


<pre><code><b>use</b> <a href="../dependencies/stylus/event.md#stylus_event">stylus::event</a>;
<b>use</b> <a href="../dependencies/stylus/object.md#stylus_object">stylus::object</a>;
<b>use</b> <a href="../dependencies/stylus/transfer.md#stylus_transfer">stylus::transfer</a>;
<b>use</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context">stylus::tx_context</a>;
</code></pre>



<a name="hello_world_dog_walker_IWalkTheDog"></a>

## Struct `IWalkTheDog`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/dog_walker.md#hello_world_dog_walker_IWalkTheDog">IWalkTheDog</a> <b>has</b> <b>copy</b>, drop
</code></pre>



<details>
<summary>Fields</summary>


<dl>
</dl>


</details>

<a name="hello_world_dog_walker_CanWalkDogCap"></a>

## Struct `CanWalkDogCap`



<pre><code><b>public</b> <b>struct</b> <a href="../hello_world/dog_walker.md#hello_world_dog_walker_CanWalkDogCap">CanWalkDogCap</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: <a href="../dependencies/stylus/object.md#stylus_object_UID">stylus::object::UID</a></code>
</dt>
<dd>
</dd>
</dl>


</details>

<a name="hello_world_dog_walker_create"></a>

## Function `create`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/dog_walker.md#hello_world_dog_walker_create">create</a>(ctx: &<b>mut</b> <a href="../dependencies/stylus/tx_context.md#stylus_tx_context_TxContext">stylus::tx_context::TxContext</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/dog_walker.md#hello_world_dog_walker_create">create</a>(ctx: &<b>mut</b> TxContext) {
    transfer::transfer(
        <a href="../hello_world/dog_walker.md#hello_world_dog_walker_CanWalkDogCap">CanWalkDogCap</a> { id: object::new(ctx) },
        tx_context::sender(ctx)
    );
}
</code></pre>



</details>

<a name="hello_world_dog_walker_walk_the_dog"></a>

## Function `walk_the_dog`



<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/dog_walker.md#hello_world_dog_walker_walk_the_dog">walk_the_dog</a>(_: &<a href="../hello_world/dog_walker.md#hello_world_dog_walker_CanWalkDogCap">hello_world::dog_walker::CanWalkDogCap</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>entry</b> <b>fun</b> <a href="../hello_world/dog_walker.md#hello_world_dog_walker_walk_the_dog">walk_the_dog</a>(_: &<a href="../hello_world/dog_walker.md#hello_world_dog_walker_CanWalkDogCap">CanWalkDogCap</a>) {
    emit(<a href="../hello_world/dog_walker.md#hello_world_dog_walker_IWalkTheDog">IWalkTheDog</a> { });
}
</code></pre>



</details>
