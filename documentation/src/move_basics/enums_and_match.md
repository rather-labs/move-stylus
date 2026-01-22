# Enums and Match

An `enum` in Move is a user-defined type that can represent one of several variants. Each variant can optionally hold associated data. Enums are useful for modeling data that can take on different forms. Recursive enums, where a variant can contain another instance of the same enum type, are **not** supported.

## Definition

An enum is defined using the `enum` keyword followed by the enum name and its variants. Each variant can have associated data types. Enums, like structs, can have abilities such as `copy`, `drop`, and `store`. Enums must have at least one variant.

```move
public enum Shape has copy, drop {
    Dot,
    Circle(u8),
    Rectangle { width: u8, height: u8 },
}
```

In the code example above, we define an enum `Shape` with three variants:
- `Dot` with no associated data.
- `Circle` that holds a single `u8` value representing the radius.
- `Rectangle` that holds two named fields: `width` and `height`.

## Instantiating


Enums are *internal* to the module in which they are defined. They can only be constructed, read, and unpacked within that same module.

[Similar to structs](./string.md), enums are instantiated by specifying the enum type, selecting a variant, and providing values for any fields associated with that variant.

```move
let dot = Shape::Dot;
let circle = Shape::Circle(5);
let rectangle = Shape::Rectangle { width: 10, height: 20 };
```

Depending on the requirements of your application, enums can either expose **public constructors** for external use or be instantiated **privately within the defining module** as part of the internal logic.


## Using in Type Definitions

The primary advantage of enums is their ability to encapsulate different data structures within a single type. For example, consider a struct that holds a vector of `Shape` values:

```move
public struct ShapeCollection(vector<Shape>) has copy, drop;

let shapes = ShapeCollection(vector[
    Shape::Dot,
    Shape::Circle(5),
    Shape::Rectangle { width: 10, height: 20 },
]);
```

All variants of the `Shape` enum share the same type—`Shape`—which enables the creation of a homogeneous vector containing multiple variants. This level of flexibility is not possible with structs, since each struct defines a single, fixed structure.

## Pattern Matching with `match`

Pattern matching allows you to destructure enums and execute different code paths based on the variant. It is similar to a switch-case statement found in other programming languages but is more powerful due to its ability to bind variables to associated data.

Pattern matching enables logic to be executed based on the structure or variant of a value. It is expressed using the `match` construct, which takes the value to be matched in parentheses, followed by a block of match arms.
Each arm specifies a pattern and the corresponding expression to run when that pattern is satisfied.

```move
public fun is_dot(shape: Shape): bool {
    match (shape) {
        Shape::Dot => true,
        Shape::Circle(_) => false,
        Shape::Rectangle { width: _, height: _ } => false,
    }
}
```

The `match` keyword evaluates the `shape` parameter and compares it against each pattern in the match arms. When a pattern matches, the corresponding expression is executed. In this example, if `shape` is a `Dot`, the function returns `true`; otherwise, it returns `false`.

For variants with associated data, you can use `_` to ignore the data if it is not needed, or you can bind it to a variable for use within the expression.

### The *any* condition

In situations where you want to match any variant without caring about its specific type or associated data, you can use the wildcard pattern `_`. This pattern matches any value and is useful for providing a default case.

```move
public fun is_dot(shape: Shape): bool {
    match (shape) {
        Shape::Dot => true,
        _ => false, // Matches any other variant
    }
}
```

In certain scenarios—such as matching against primitive values or collections like vectors—it may be impractical to enumerate every possible case. In those scenarios, the wildcard pattern `_` can be employed to match any value that does not explicitly match previous patterns.

```move
public fun is_circle_with_positive_radius(shape: Shape): bool {
    match (shape) {
        Shape::Circle(0) => false,
        Shape::Circle(_) => true,
        _ => false, // Matches Dot and Rectangle variants
    }
}
```


<!-- TODO: add the try_into pattern -->
