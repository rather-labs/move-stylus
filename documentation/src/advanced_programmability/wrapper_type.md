# Pattern: Wrapper Type

Sometimes it is useful to define a new type that behaves like an existing one but with specific modifications or restrictions. For instance, you might design a collection type that functions like a vector but prevents modification of elements once they are inserted. The *wrapper type* pattern is a practical way to achieve this.

#### Definition
The wrapper type pattern is a design approach where a new type is created to wrap an existing type. While distinct from the original, the wrapper type can be converted to and from it.

In most cases, it is implemented as a positional struct containing a single field.

```move
module book::stack;

/// Very simple stack implementation using the wrapper type pattern. Does not allow
/// accessing the elements unless they are popped.
public struct Stack<T>(vector<T>) has copy, store, drop;

/// Create a new instance by wrapping the value.
public fun new<T>(value: vector<T>): Stack<T> {
    Stack(value)
}

/// Push an element to the stack.
public fun push_back<T>(v: &mut Stack<T>, el: T) {
    v.0.push_back(el);
}

/// Pop an element from the stack. Unlike `vector`, this function won't
/// fail if the stack is empty and will return `None` instead.
public fun pop_back<T>(v: &mut Stack<T>): Option<T> {
    if (v.0.length() == 0) option::none()
    else option::some(v.0.pop_back())
}

/// Get the size of the stack.
public fun size<T>(v: &Stack<T>): u64 {
    v.0.length()
}
```

## Common Practices

When the goal is to extend the behavior of an existing type, it is common to provide accessors for the wrapped type. This approach allows users to interact with the underlying type directly when necessary.

For example, the following code defines the `inner()`, `inner_mut()`, and `into_inner()` methods for the `Stack` type:

```move
/// Allows reading the contents of the `Stack`.
public fun inner<T>(v: &Stack<T>): &vector<T> {
    &v.0
}

/// Allows mutable access to the contents of the `Stack`.
public fun inner_mut<T>(v: &mut Stack<T>): &mut vector<T> {
    &mut v.0
}

/// Unpacks the `Stack` into the underlying `vector`.
public fun into_inner<T>(v: Stack<T>): vector<T> {
    let Stack(inner) = v;
    inner
}
```

## Advantages

The wrapper type pattern provides several benefits:

- **Custom Functions**: Enables defining custom functions for an existing type.
- **Robust Function Signatures**: Restricts function signatures to the new type, making the code more reliable.
- **Improved Readability**: Offers clearer, more descriptive type names, enhancing code readability.

## Disadvantages

The wrapper type pattern is particularly useful in two scenarios:
- When limiting the behavior of an existing type while exposing a custom interface.
- When extending the behavior of an existing type.

However, it comes with some drawbacks:

- **Verbosity**: Implementation can be verbose, especially if many methods of the wrapped type need to be exposed.
- **Sparse Implementation**: Often minimal, as it primarily forwards calls to the wrapped type.

