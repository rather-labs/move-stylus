# Modules

A module is the fundamental unit of code organization in Move. It provides a mechanism to group related functionality and enforce isolation. By default, all members within a module are private, ensuring encapsulation and controlled access. In this section, we will cover how to define a module, declare its members, and reference it from other modules.

## Module Declaration

Modules in Move are declared using the module keyword, followed by the package address, the module name, a semicolon, and the module body. Module names must follow the snake_case convention—lowercase letters with underscores separating words—and must be unique within the package.

```move
module <package_address>::<module_name>;
```

If you need to declare more than one module in a file, you must use module block syntax.

```move
module <package_address>::<module_name> {
    // module body
}
```

Typically, each file in the `sources/` directory defines a single module. The file name must correspond to the module name; for instance, a `counter` module should reside in a file named `counter.move`.


[Structs](./move_basics/structs.md), [Functions](./move_basics/functions.md), [Constants](./move_basics/constants.md) and [Imports](./move_basics/imports) are all declared within the module body.


## Module Members

Module members are defined within the body of a module. They can include data structures, functions, and constants. The example below demonstrates a simple module that declares a `struct`, a `function`, and a `const` value:

```move
module book::counter;

const INITIAL_COUNT: u64 = 0;

public struct Counter has drop {
    value: u64
}

fun create(): Counter {
    Counter { value: INITIAL_COUNT }
}

/// Increment a counter by 1.
fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}


/// Read counter.
fun read(counter: &Counter): u64 {
    counter.value
}
```

## Address and Named Address

A module address in Move can be specified in two ways:

- **Address literal** — written directly, without requiring the `@` prefix.
    ```move
    module 0x1::my_module;
    ```
- **Named address** — defined in the `[addresses]` section of the [Package Manifest](./concepts/manifest.md).
    ```move
    module book::my_module;
    ```

For example, both forms below resolve to the same value because the `Move.toml` includes the entry:

```toml
[addresses]
book = "0x0"
```

