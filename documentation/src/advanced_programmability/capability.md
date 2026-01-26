# Pattern: Capability

In programming, a **capability** is a specialized token that grants its owner the explicit right to perform a specific action or access a protected resource. This pattern is a powerful way to manage security and access control. Instead of checking a user's name against a list (Identity-Based Access Control), the system simply checks if the caller is "holding" the required capability.

## Capability as an Object

In the [Object Model](../object_model), capabilities are represented as objects. An owner of an object can pass this object to a function to prove that they have the right to perform a specific action. Due to strict typing, the function taking a capability as an argument can only be called with the correct capability.

In the example below, we define an `AdminCap`. Upon deployment, the `init` function creates this object and transfers it to the caller. Subsequently, only the holder of this unique `AdminCap` can execute `admin_cap_fn`.

```move
module test::capability;

use stylus::{transfer::{Self}, object::{Self, UID}, tx_context::{Self, TxContext}};

public struct AdminCap has key { id: UID }

/// Create the AdminCap object on module deployment and transfer it to the caller
fun init(ctx: &mut TxContext) {
    transfer::transfer(
        AdminCap { id: object::new(ctx) },
        ctx.sender()
    )
}

entry fun admin_cap_fn(_: &AdminCap ) {}
```

## Address Check vs Capability

Utilizing objects as capabilities is a relatively new concept in blockchain programming. And in other smart-contract languages, authorization is often performed by checking the address of the sender. This pattern is still viable on our framework, however, overall recommendation is to use capabilities for better security, discoverability, and code organization.

Using capabilities has several advantages over the address check:

* Migration of admin rights is easier with capabilities due to them being objects. In case of address, if the admin address changes, all the functions that check the address need to be updated - hence, require a package upgrade.
* Function signatures are more descriptive with capabilities. 
* Object Capabilities don't require extra checks in the function body, and hence, decrease the chance of a developer mistake.
* An owned Capability also serves in discovery. The owner of the `AdminCap` can see the object in their account (via a Wallet or Explorer), and know that they have the admin rights. This is less transparent with the address check.

However, the address approach has its own advantages. For example, if an address is multisig, and transaction building gets more complex, it might be easier to check the address. Also, if there's a central object of the application that is used in every function, it can store the admin address, and this would simplify migration. The central object approach is also valuable for revocable capabilities, where the admin can revoke the capability from the user.