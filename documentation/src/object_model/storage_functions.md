# Storage Functions

The [Stylus Framework](./../stylus_framework) provides a set of built-in functions within the `transfer` module to define and manage object ownership:

1. `transfer::transfer`: sends an object to a specific address, placing it in an **address-owned** state.
2. `transfer::freeze_object`: transitions an object into an immutable state. It becomes a public constant and can never be modified.
3. `transfer::share_object` - transitions an object into a shared state, making it accessible to all users.

## Transfer

The `transfer::transfer` function is used to send an object to a specific address. It's signature is as follows:

```move
module stylus::transfer;

// Transfer `obj` to `recipient`.
public fun transfer<T: key>(obj: T, recipient: address);

```

It only accepts a type with the [key ability](./ability_key.md) and the [address](../move_basics/address_type.md) of the recipient. Note that the function is generic over type `T`, which represents the type of the object being transferred. The object is passed into the function _by value_; it is moved into the function's scope and then moved to the recipient's address.

In the following example, you can see how it can be used in a module that defines and sends an object to the transaction sender.

```move
module book::transfer_to_sender;

/// A struct with `key` is an object. The first field is `id: UID`!
public struct AdminCap has key { id: UID }

/// `init` function is a special function that translated as the equivalent
/// of the constructor function in Solidity
entry fun init(ctx: &mut TxContext) {
    // Create a new `AdminCap` object, in this scope.
    let admin_cap = AdminCap { id: object::new(ctx) };

    // Transfer the object to the transaction sender.
    transfer::transfer(admin_cap, ctx.sender());
}

/// Transfers the `AdminCap` object to the `recipient`. Thus, the recipient
/// becomes the owner of the object, and only they can access it.
public fun transfer_admin_cap(cap: AdminCap, recipient: address) {
    transfer::transfer(cap, recipient);
}
```

When the module is deployed, the `init` function must be called (the module [_constructor_](./../evm_specifics/constructor.md)) to initialize the contract. The `AdminCap` object which we created in it will be transferred to the transaction sender. The `ctx.sender()` function returns the sender address for the current transaction.

Once the `AdminCap` has been transferred to the sender, for example, to `0xa11ce`, the sender, and only the sender, will be able to access the object.

## Freeze

The `transfer::freeze_object` function is used to put an object into an _immutable_ state. Once an object is _frozen_, it can never change, and it can be accessed by anyone by immutable reference.

```move
module stylus::transfer;

// Make object immutable and allow anyone to read it.
public fun freeze_object<T: key>(obj: T);
```

The function only accepts a generic type `T` with the `key` ability. Just like all other storage functions, it takes the object _by value_.

Let's extend the previous example and add a function that allows the admin to create a `Config` object and freeze it:

```move
/// Some `Config` object that the admin can `create_and_freeze`.
public struct Config has key {
    id: UID,
    message: String
}

/// Creates a new `Config` object and freezes it.
public fun create_and_freeze(
    _: &AdminCap,
    message: String,
    ctx: &mut TxContext
) {
    let config = Config {
        id: object::new(ctx),
        message
    };

    // Freeze the object so it becomes immutable.
    transfer::freeze_object(config);
}

/// Returns the message from the `Config` object.
/// Can access the object by immutable reference!
public fun message(c: &Config): String { c.message }
```

`Config` is an object that has a message field, and the `create_and_freeze` function creates a new `Config` and freezes it. Once the object is frozen, it can be accessed by anyone by immutable reference. The message function is a public function that returns the `message` from the `Config` object. `Config` is now publicly available by its `ID`, and the message can be read by anyone.

## Share

The `transfer::share_object` function is used to put an object into a _shared_ state. Once an object is _shared_, it can be accessed by anyone by a *mutable* reference (hence, immutable too). The `transfer::share_object` function is used to put an object into a _shared_ state. Once an object is _shared_, it can be accessed by anyone by a *mutable* reference (hence, immutable too).

This means it does not make sense to pass a shared object _by value_, since you can't do anything with it (cannot be transfered, and if you try to unpack its values, the compiler throw an error because the id field is not handled).

The function signature is as follows, only accepts a type with the key ability:

```move
module stylus::transfer;

/// Put an object to a Shared state - can be accessed mutably and immutably.
public fun share_object<T: key>(obj: T);
```

It is important to note that sharing an object is a terminal state for its accessibility: once shared, an object can no longer be transferred or frozen. However, it can still be deleted. This creates an exception to the _by value_ rule—a shared object can only be passed by value if the receiving function explicitly consumes (deletes) it:

```move
/// Creates a new `Config` object and shares it.
public fun create_and_share(message: String, ctx: &mut TxContext) {
    let config = Config {
        id: object::new(ctx),
        message
    };

    // Share the object so it becomes shared.
    transfer::share_object(config);
}

/// Deletes the `Config` object, takes it by value.
/// Can be called on a shared object!
public fun delete_config(c: Config) {
    let Config { id, message: _ } = c;
    id.delete()
}

// Won't work!
public fun transfer_config(c: Config, to: address) {
    transfer::transfer(c, to);
}
```

## Recap

1. Transfer
    * `transfer::transfer` is used to send an object to an address
    * The object becomes address owned and can only be accessed by the recipient
    * Address owned object can be used by reference or by value, including being transferred to another address

2. Freeze
    * `transfer::freeze_object`function is used to put an object into an immutable state
    * Once an object is frozen, it can never be changed, deleted or transferred, and it can be accessed by anyone by immutable reference

3. Share
    * `transfer::share_object` function is used to put an object into a shared state
    * Once an object is shared, it can be accessed by anyone by a mutable reference
    * Shared objects can be deleted, but they can't be transferred or frozen

