# What is an Object?

The _Object Model_ can be viewed as a high-level abstraction representing digital assets as objects. These objects have their own type and associated behaviors, a unique identifier, and support native storage operations like _transfer_, _share_ and _freeze_. Designed to be intuitive and easy to use, the _Object Model_ enables a wide range of use cases to be implemented with ease.

Objects have the following properties:

* **Type**: Every object has a type, defining the structure and behavior of the object. Objects of different types cannot be mixed or used interchangeably, ensuring objects are used correctly according to their type system.

* **Unique ID**: Each object has a unique identifier, distinguishing it from other objects. This ID is generated upon the object's creation and is immutable. It's used to track and identify objects within the system.

* **Owner**: Every object is associated with an owner (which might be an address or [_another object_](./wrapped_objects.md)), who has control over changes to the object. [Ownership](./ownership.md) can be exclusive, shared across the network, or frozen, allowing read-only access without modification or transfer capabilities.

* **Data**: Objects encapsulate their data, simplifying management and manipulation. The data structure and operations are defined by the object's type.

>[!Warning]
Ownership does not control the confidentiality of an object — it is always possible to read the contents of an on-chain object from outside of Move. You should never store unencrypted secrets inside of objects.

> [!Note]
> Certain kind of object have a pre-computed ID called [`NamedId`](./named_ids.md).
> It is possible to have two distinct objects with the same `NameId`. Those cases should be handled with care.
