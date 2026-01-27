# Abilities: Copy

In Move, the _copy_ ability on a type indicates that the instance or the value of the type can be copied, or duplicated. While this behavior is provided by default when working with numbers or other primitive types, it is not the default for custom types. Move is designed to express digital assets and resources, and controlling the ability to duplicate resources is a key principle of the resource model. However, the Move type system allows you to add the _copy_ ability to custom types:

```move
public struct Copyable has copy {}
```

In the example above, we define a custom type `Copyable` with the _copy_ ability. This means that instances of `Copyable` can be copied, both implicitly and explicitly.

```move
let a = Copyable {}; // allowed because the Copyable struct has the `copy` ability
let b = a;   // `a` is copied to `b`
let c = *&b; // explicit copy via dereference operator

// Copyable doesn't have the `drop` ability, so every instance (a, b, and c) must
// be used or explicitly destructured. The `drop` ability is explained below.
let Copyable {} = a;
let Copyable {} = b;
let Copyable {} = c;
```

In the example above, `a` is copied to `b` implicitly, and then explicitly copied to `c` using the dereference operator. If `Copyable` did not have the _copy_ ability, the code would not compile, and the Move compiler would raise an error.

>[!Note]
In Move, destructuring with empty brackets is often used to consume unused variables, especially for types without the drop ability.
This prevents compiler errors from values going out of scope without explicit use. Also, Move requires the type name in destructuring (e.g., `Copyable` in `let Copyable {} = a;`) because it enforces strict typing and ownership rules.

## Copying and Drop

The _copy_ ability is closely related to the [drop ability](./ability_drop.md). If a type has the _copy_ ability, it is very likely that it should have _drop_ too. This is because the _drop_ ability is required to clean up resources when the instance is no longer needed. If a type only has _copy_, managing its instances gets more complicated, as the instances must be explicitly used or consumed.

```move
public struct Value has copy, drop {}
```

All of the primitive types in Move behave as if they have the _copy_ and _drop_ abilities. This means that they can be copied and dropped, and the Move compiler will handle the memory management for them.

All native types in Move have the copy ability. This includes:

* [bool](./primitive_types.md)
* [unsigned integers](./primitive_types.md)
* [vector](./vector.md)
* [address](./address.md)
