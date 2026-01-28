# Package Manifest

The `Move.toml` is a manifest file that describes the [package](./packages.md) and its dependencies. It is written in TOML format and contains multiple sections, the most important of which are `[package]`, `[dependencies]` and `[addresses]`.

```toml
[package]
name = "my_project"
version = "0.0.0"
edition = "2024"

[dependencies]
Example = { git = "https://github.com/example/example.git", subdir = "path/to/package", rev = "framework/testnet" }

[addresses]
std =  "0x1"

[dev-addresses]
alice = "0xB0B"
```

## Sections

### Package
The `[package]` section is used to describe the package. None of the fields in this section are published on chain, but they are used in tooling and release management.

* `name` - the name of the package when it is imported;
* `version` - the version of the package, can be used in release management;

### Dependencies
The `[dependencies]` section is used to specify the dependencies of the project. Each dependency is specified as a key-value pair, where the key is the name of the dependency, and the value is the dependency specification. The dependency specification can be a git repository URL or a path to the local directory.

```
# git repository
Example = { git = "https://github.com/example/example.git", subdir = "path/to/package", rev = "framework/testnet" }

# local directory
StylusFramework = { local = "../stylus-framework/" }
```

Packages also import addresses from other packages. For example, the Sui dependency adds the std and sui addresses to the project. These addresses can be used in the code as aliases for the addresses.

### Dev-dependencies
<!--

TODO: try if this is supported
-->

### Resolving Version Conflicts with Override

Sometimes dependencies have conflicting versions of the same package. For example, if you have two dependencies that use different versions of the Example package, you can override the dependency in the `[dependencies]` section. To do so, add the *override* field to the dependency. The version of the dependency specified in the `[dependencies]` section will be used instead of the one specified in the dependency itself.

```
[dependencies]
Example = { override = true, git = "https://github.com/example/example.git", subdir = "crates/sui-framework/packages/sui-framework", rev = "framework/testnet" }
```

### Addresses

The `[addresses]` section is used to add aliases for the addresses. Any address can be specified in this section, and then used in the code as an alias. For example, if you add `alice = "0xA11CE"` to this section, you can use alice as `0xA11CE` in the code.

### Dev-addresses
<!--

TODO: try if this is supported
-->
