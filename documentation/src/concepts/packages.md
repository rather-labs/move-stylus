# Package

In Move, smart contracts are organized into Packages. A contract is the primary unit of deployment; once uploaded to the blockchain, it is assigned a unique, immutable address that others can use to call its functions.

A package acts as a container for modules (i.e. contracts), which serve as distinct namespaces for defining types (structs) and logic (functions).


```
package 0x...
    module a
        struct A1
        fun hello_world()
    module b
        struct B1
        fun hello_package()
```
## Package Structure

Locally, a package is a directory with a `Move.toml` file and a `sources` directory. The `Move.toml` file - called the "package manifest" - contains metadata about the package, and the sources directory contains the source code for the modules. Package usually looks like this:

```
sources/
    my_module.move
    another_module.move
    ...
tests/
    ...
examples/
    using_my_module.move
Move.toml
```

## Package Address

Each package is identified by a unique address. This address is only relevant for internal use, to distinguish and reference packages inside the project. This addresses do not represent on chain addresses. Some special packages have [reserved addresses](./address.md#seccion).
