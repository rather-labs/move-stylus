# Ability key

We already covered two out of four abilities: [Drop](../move_basics/ability_drop.md) and [Copy](../move_basics/ability_copy.md). They affect the behavior of a value in a scope and are not directly related to storage. Now it is time to cover the `key` ability, which allows a struct to be _stored_.

## Defining an Object

For a struct to be considered an object and used with storage functions, it must fulfill three strict requirements:

1.  **The `key` Ability**: The struct must be declared with `has key`.
2.  **The `id` Field**: The very first field in the struct **must** be named `id` and have the type [`UID`](./uid_and_id.md) or [`NamedId`](./named_ids.md).
3.  **The `store` Requirement**: All other fields within the struct must have the `store` ability.

```move
/// `User` object definition.
public struct User has key {
    id: UID,
    name: String, // field types must have `store`
}

/// Creates a new instance of the `User` type.
/// Uses the special struct `TxContext` to derive a Unique ID (UID).
public fun new(name: String, ctx: &mut TxContext): User {
    User {
        id: object::new(ctx), // creates a new UID
        name,
    }
}
```

>[!Note]
`UID` is a type that does not have the `drop` or `copy` abilities. Because every object contains a `UID`, **objects themselved can never be dropped or copied**. This ensures that assets cannot be accidentally deleted or duplicated, enforcing strict scarcity and accountability.

## Types with the key Ability

Due to the `UID` or `NamedId` requirement for types with `key`, none of the native types in Move can have the `key` ability, nor can any of the types in the [Standard Library](./../move_basics/standard_library.md). The `key` ability is present only in some [Stylus Framework](./../stylus_framework) types and in custom types.

## Summary

* The `key` ability defines an object
* The first field of an object must be id with type `UID`
* Fields of a `key` type must the have `store` ability
* Objects cannot have `drop` or `copy`

The key ability defines objects in Move and forces the fields to have store. In the next section we cover the [store](./ability_store.md) ability to later explain how storage operations work.
