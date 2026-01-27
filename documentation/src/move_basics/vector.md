# Vector

Vectors are the built-in mechanism for storing collections of elements in Move. They resemble arrays in other programming languages, but with some key differences. This section provides an introduction to the vector type and its operations.

## Syntax

The `vector` type is declared using the `vector` keyword followed by the element type in angle brackets. Elements can be any valid Move type, including other vectors.

Move also provides a vector literal syntax: you can construct vectors with the `vector` keyword followed by square brackets containing the elements, or leave the brackets empty to create an empty vector.

```move
// A vector of unsigned 64-bit integers
let v1: vector<u64> = vector[1, 2, 3];

// A nested vector of booleans
let v2: vector<vector<bool>> = vector[vector[true, false], vector[false, true]];

// An empty vector of bytes
let empty_vec: vector<u8> = vector[];
```

The `vector` type is a built-in type in Move, so you do not need to import any modules to use it. Vector operations are defined in the `std::vector` module of the [Standard Library](./standard_library.md), which is implicitly imported and can be used directly without explicit `use` import.

## Operations

The standard library offers several methods for working with vectors. Some of the most commonly used operations include:

- **push_back**: Appends an element to the end of the vector.
- **pop_back**: Removes the last element from the vector.
- **length**: Returns the total number of elements in the vector.
- **is_empty**: Returns `true` if the vector contains no elements.
- **remove**: Deletes the element at a specified index.

```move
let mut v: vector<u64> = vector[];

// Adding elements to the vector
v.push_back(10);
v.push_back(20);

let len = v.length();
assert_eq!(len, 2);
assert_eq!(v.is_empty(), false);

let last_element = v.pop_back();
assert_eq!(last_element, 20);
```

## Destroying a Vector of non-droppable types

A vector containing non-droppable types cannot be discarded. If you create a vector of types that lack the drop ability, the value must be handled explicitly. When such a vector is empty, the compiler enforces an explicit call to the `destroy_empty` function.

```move
struct NonDroppable { }

fun destroy_vector_of_non_droppable() {
    let mut v: vector<NonDroppable> = vector[];
    // Perform operations on the vector...

    // Explicitly destroy the empty vector
    v.destroy_empty();
}
```

The `destroy_empty` function will fail at runtime if you call it on a non-empty vector.
