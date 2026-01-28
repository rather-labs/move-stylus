# Abilities introduction

Move's type system supports abilities, which define the behaviors that instances of a type are permitted to perform. They are specified directly in the struct.

In the previous section, we introduced struct definitions and demonstrated how to create and work with them. Notice that instances of the `Author` and `Book` structs had to be unpacked for the code to compile. This is the default behavior of a struct without any declared abilities.

> [!NOTE]
> Throughout this manual, you will encounter chapters titled `Ability: <name>`, where `<name>` refers to a specific ability. Each of these chapters provides a detailed explanation of the ability.

## Syntax

Abilities are declared in the struct definition using the `has` keyword, followed by a comma-separated list of abilities. The syntax is as follows:

```move
struct <StructName> has <ability1>, <ability2>, ... {
    // struct fields
}
```

Where:
- `<StructName>` is the name of the struct being defined.
- `<ability1>, <ability2>, ...` are the abilities assigned to the struct.


Move supports the following abilities:
- `copy` — allows the struct to be duplicated. Explained in [Ability: copy](./ability_copy.md).
- `drop` — allows the struct to be discarded without being used. Explained in [Ability: drop](./ability_drop.md).
- `key` — allows the struct to be stored in storage. Explained in [Ability: key](./../object_model/ability_key.md).
- `store` — allows the struct to be stored in structs with the `key` ability. Explained in [Ability: store](./../object_model/ability_store.md).


# No abilities

By default, if no abilities are specified, the struct has none of the abilities. This means the struct cannot be copied, discarded, or stored in strorage. They can only be passed around and requires special handling to use them. We call those structs *Hot Potato*, which is a powerful pattern discussed in more detail in [Hot Potato Pattern](./../advanced_programmability/hot_potato.md) chapter.


