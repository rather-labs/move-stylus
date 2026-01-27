# Importing Modules

 Move supports modularity and code reuse through module imports. Modules within the same package can import each other, and new packages can depend on existing ones to access their modules. This section explains the basics of importing modules and using them in your code.

 ## Importing a module

 Modules defined in the same package can be imported using the `use` keyword followed by the module's fully qualified name. The syntax is as follows:

 ```
 use <package_address>::<module_name>;
 ```

#### Example

```move
// File: sources/counter.move

module book::counter;

const INITIAL_COUNT: u64 = 0;

public struct Counter has drop {
    value: u64
}

public fun create(): Counter {
    Counter { value: INITIAL_COUNT }
}

/// Increment a counter by 1.
public fun increment(counter: &mut Counter) {
    counter.value = counter.value + 1;
}

/// Read counter.
public fun read(counter: &Counter): u64 {
    counter.value
}
```

Another module defined in the same package can import and use the `counter` module as follows:

```move
// File: sources/main.move

module book::main;

// Importing the counter module from the book package
use book::counter;

fun main() {
    // Creating a new Counter instance using the create function from the counter module
    let my_counter = counter::create();

    // Incrementing the counter
    counter::increment(&mut my_counter);

    // Reading the current value of the counter
    let value = counter::read(&my_counter);
}
```

In this example, we import the `counter` module from the `book` package. We then use the functions defined in the `counter` module to create, increment, and read a counter.

> [!NOTE]
> To import an item (struct, function, constant, etc.) from another module, it must be declared with the `public` keyword (or `public(package)`—see [visibility modifiers](./visibility_modifiers.md)). For instance, the `Counter` struct and the `create` function in `counter` module are marked `public`, allowing them to be accessed in `main`.

## Importing members

You can also import specific members from a module using the `use` keyword followed by the fully qualified name of the member. The syntax is as follows:

```
use <package_address>::<module_name>::<member_name>;
```

#### Example

```move
// File: sources/main.move

module book::main;

// Importing specific members from the counter module
use book::counter::{create, increment, read, Counter};

fun main(): Counter {
    let my_counter = create();
    increment(&mut my_counter);
    let value = read(&my_counter);
    my_counter
}
```

Imports can either be grouped using curly braces `{}` as shown above, or declared individually:

```move
use book::counter::create;
use book::counter::increment;
use book::counter::read;
use book::counter::Counter;
```

> [!NOTE]
> Importing individual function names in Move is uncommon, as overlapping names can lead to ambiguity. A better practice is to import the full module and call functions using the module path. Types, however, have unique identifiers and are best imported separately.

You can use the `Self` keyword in a group import to bring in both the module itself and its members. `Self` refers to the module as a whole, allowing you to import the module alongside its contents.

```move
// File: sources/main.move

module book::main;

// Importing specific members from the counter module
use book::counter::{Self, Counter};

fun main(): Counter {
    let my_counter = counter::create();
    counter::increment(&mut my_counter);
    let value = counter::read(&my_counter);
    my_counter
}
```

## Resolving Name Conflicts

When importing modules or members, name conflicts can arise if two imported items share the same name. To resolve such conflicts, Move allows you to use the `as` keyword to create an alias for the imported item.

```move
// File: sources/main.move

module book::main;

// Importing specific members from the counter module
use book::counter::{Self as count, Counter as Count};

fun main(): Count {
    let my_counter = count::create();
    count::increment(&mut my_counter);
    let value = count::read(&my_counter);
    my_counter
}
```

## Adding an External Dependency

To use modules from an external package, you need to declare the dependency in your [manifest](../concepts/manifest.md) file.

```toml
[dependencies]
Remote = { git = "https://github.com/example/example-stylus.git", rev = "main", subdir = "packages/remote" }
Local = { local = "../local_packages/local" }
```

The `[dependencies]` section lists each package dependency. The entry key is the package name (e.g., `Remote` or `Local`), and the value is either a Git import or a local path. A Git import specifies the package URL, the subdirectory containing the package, and the revision, while a local path points to the relative directory of the package.

When you add a dependency, all of its own dependencies are also made available to your package. If a dependency is declared in the `Move.toml` file, the compiler will automatically fetch (and refetch) it during the build process.

> [!NOTE]
> The standard library and the Stylus framework are automatically included as dependencies.

# Importing Modules from Another Package

To import modules from another package, you first need to declare the dependency in your package's manifest file (`Move.toml`). Once the dependency is declared, you can import modules from that package using the `use` keyword followed by the package name and module name.

Typically, packages specify their addresses in the `[addresses]` section. Instead of writing full addresses, you can use aliases. For instance, the `std` alias is defined in the Standard Library package and serves as a shorthand for `0x1` when accessing standard library modules.

> [!NOTE]
> Module address names are defined in the `[addresses]` section of the `Move.toml` manifest, not taken from the names listed in `[dependencies]`.
