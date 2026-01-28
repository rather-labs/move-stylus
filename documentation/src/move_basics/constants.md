# Constants


Constants are immutable values defined at the module level. They provide a convenient way to assign meaningful names to static values that are reused across a module. For example, a default product price can be represented as a constant rather than hardcoding the value multiple times. Constants are embedded in the module's compiled code, and each reference to a constant results in the value being copied.

```move
const DEFAULT_PRODUCT_PRICE: u64 = 100;
const SHIOP_OWNER: address = @0xb0b;

/// Error code indicating an incorrect price
const EWrongPrice: u64 = 1;


public fun purchase(price: u64) {
    // Use the constant to check the price
    assert!(price == DEFAULT_PRODUCT_PRICE, EWrongPrice);
    // Purchase logic...
}
```

## Naming Conventions

Constants must begin with a capital letter, a rule enforced by the compiler. For constants representing values, the established convention is to use **all uppercase letters** with underscores separating words. This style ensures that constants are easily distinguishable from other identifiers in the code.

An exception applies to **error constants**, which follow the `ECamelCase` naming convention.

```move
// Regular constant
const MAX_RETRIES: u8 = 5;

// Error constant
const EInvalidOperation: u64 = 100;
```

## Immutability

Constants are immutable, meaning their values cannot be changed after they are defined. Attempting to reassign a value to a constant will result in a compilation error.

```move
const MAX_CONNECTIONS: u32 = 10;

// This will cause a compilation error
public fun change_max_connections() {
    MAX_CONNECTIONS = 20;
}
```

## Config Pattern

A typical application often requires a set of constants that are reused across the codebase. Since constants are private to the module in which they are defined, they cannot be accessed directly from other modules. A common solution is to create a dedicated **`config` module** that exposes these constants publicly, ensuring they can be referenced wherever needed while maintaining a centralized definition.

```move
module book::config;

/// Default product price
public const DEFAULT_PRICE: u64 = 100;

/// Maximum number of items allowed in a cart
public const MAX_CART_ITEMS: u64 = 50;

/// Returns the default product price
public fun default_price(): u64 {
    DEFAULT_PRICE
}

/// Returns the maximum number of items allowed in a cart
public fun max_cart_items(): u64 {
    MAX_CART_ITEMS
}
```

By exposing constants through a dedicated `config` module, other modules can import and reference them directly. This approach simplifies maintenance: if a constant value needs to be updated, only the `config` module must be modified during a package upgrade. As a result, updates are centralized, reducing duplication and ensuring consistency across the codebase.
