# Wrapped objects

Wrapping refers to nesting structs to organize data structures in Move. When an object is wrapped, the object no longer exists independently on the contract's storage. You can no longer look up the object by its `ID`, as the object becomes part of the data of the object that wraps it. Most importantly, you can no longer pass the wrapped object as an argument in a Move call. The only access point is through the object that wraps it.

It is not possible to create circular wrapping behavior, where `A` wraps `B`, `B` wraps `C`, and `C` also wraps `A`.

To embed a struct type in an object with a `key` ability, the struct type must have the `store` ability.

This example shows a basic wrapper pattern:

```move
public struct Foo has key {
    id: UID,
    bar: Bar,
}

public struct Bar has key, store {
    id: UID,
    value: u64,
}
```

In this scenario the object type `Foo` wraps the object type `Bar`. The object type `Foo` is the wrapper or wrapping object.

## Creating a wrapped object

Consider the following example:

1.  The `wrap` function takes the `Object` by value, which requires the caller to be the current owner.
2.  The `Object` is moved into the `wrapped` field of a new `Wrapper` instance.
3.  The `Wrapper` is transferred to the sender. The inner `Object` is no longer a top-level asset; it is now part of the `Wrapper`'s private state and is not directly accessible using its `ID`.

```move
module example::wrapped;

use stylus::{
    tx_context::TxContext,
    object::{Self, UID},
    transfer::{Self}
};

// The object to be wrapped
public struct Object has key, store {
    id: UID,
    data: vector<u8>
}

// The wrapper object
public struct Wrapper has key {
    id: UID,
    wrapped: Object
}

// This function takes the Object by value, wraps it in the Wrapper and transfer the Wrapper to the transaction sender.
public fun wrap(o: Object, ctx: &mut TxContext) {
    transfer::transfer(Wrapper { id: object::new(ctx), wrapped: o }, ctx.sender());
}
```

## Unwraping a wrapped object

You can take out the wrapped object and transfer it to an address, modify it, delete it, or freeze it. This is called unwrapping. When an object is unwrapped, it becomes an independent object again and can be accessed directly by its `ID`. The object's `ID` stays the same across wrapping and unwrapping.

>[!Note]
The wrapped object cannot be extracted without destroying the wrapper!

This example shows a basic function used to unwrap an object:

```move
/// Unpacks the Wrapper, deletes its UID, and returns the inner Object to the sender.
public fun unwrap(w: Wrapper, ctx: &TxContext) {
    // Unpack the struct to access the fields
    let Wrapper { id, wrapped } = w;

    // The Wrapper's UID must be explicitly deleted
    id.delete();

    // The inner Object is now a top-level asset again
    transfer::transfer(wrapped, ctx.sender());
}
```

If a user calls `wrap` followed by `unwrap`, the final state returns the original `Object` to the user's address as a standalone entity.
