# Ownership

There exist four distinct ownership types for objects: account-owned, shared, immutably shared (_a.k.a_ frozen), and object-owned (_a.k.a_ wrapped). Each model offers unique characteristics and suits different use cases, enhancing flexibility and control in object management.

### Account Owned

The account-owned is the foundational ownership type. The object is owned by a single account, granting that account exclusive control over the object within the behaviors associated with its type. This model embodies the concept of true ownership, where the account possesses complete authority over the object, making it inaccessible to others for modification or transfer. Therefore, no one can use your assets unless authorized by you.

### Shared State

A shared object is a public, mutable object that is accessible to anyone on the network. They are designed such that multiple users or smart contracts can interact with the same object concurrently. Shared objects can be read and modified by any account, and the rules of interaction are defined by the implementation of the object. Typical uses for shared objects are: marketplaces, shared resources, escrows, and other scenarios where multiple accounts need access to the same state.

### Immutably Shared or Frozen State

A frozen (immutably shared) object becomes permanently read-only. These immutable objects, while readable, cannot be modified or moved, providing a stable and constant state accessible to all network participants. Frozen objects are ideal for public data, reference materials, and other use cases where the state permanence is desirable.

### Object Owned or Wrapped Objects

Wrapping refers to nesting objects to organize data structures in Move. When an object is wrapped, the object no longer exists independently on-chain. You can no longer look up the object by its ID, as the object becomes part of the data of the object that wraps it.  This feature allows creating complex relationships between objects, storing large heterogeneous collections, and implementing extensible and modular systems. Practically speaking, since the transactions are initiated by accounts, the transaction still accesses the parent object, but it can then access the child objects through the parent object.

A practical use case is a game character. Alice can own the Hero object from a game, and the Hero can own items: also represented as objects, like a "Map", or a "Compass". Alice may take the "Map" from the "Hero" object, and then send it to Bob, or sell it on a marketplace. With object owner, it becomes very natural to imagine how the assets can be structured and managed in relation to each other.


## Summary

* Account Owned: Objects are owned by a single account, granting exclusive control over the object.
* Shared: Objects can be shared with the network, allowing multiple accounts to read and modify the object.
* Frozen: Objects become permanently read-only, providing a stable and constant state.
* Object Owned: Objects can own other objects, enabling complex relationships and modular systems.