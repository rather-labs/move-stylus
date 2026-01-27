# Functions

Functions form the foundation of Move programs. They can be invoked from user transactions or other functions, and they organize executable code into reusable units. A function may accept arguments and return a value. Functions are declared at the module level using the `fun` keyword. By default, like other members of a module, they are private and accessible only within the defining module.

```move
module book::math;

public fun add(a: u64, b: u64): u64 {
    a + b
}

#[test]
public fun test_add() {
    let result = add(2, 3);
    // result now holds the value 5
    assert_eq!(result, 5);
}
```

In this example, a function `add` is defined that accepts two arguments of type `u64` and returns their sum. The `test_add` function, located within the same module, serves as a test by invoking `add`. The test relies on the `assert_eq!` macro to check whether the result of `add` matches the expected value. If the condition inside `assert!` evaluates to false, execution is automatically aborted.

## Function declaration

Functions are declared using the `fun` keyword, followed by the function name, a list of parameters enclosed in parentheses, an optional return type, and a block of code enclosed in curly braces. The last expression in the function body is treated as the return value.

```move
fun function_name(param1: Type1, param2: Type2): ReturnType {
    // function body
    // last expression is the return value
}
```

> [!NOTE]
> In Move, functions are typically named using `snake_case`, where words are separated by underscores (e.g., `my_function_name`).

## Accesing functions

### Public functions

Like other members of a module, functions can be imported and accessed through a path. This path is composed of the module path followed by the function name, separated with `::`. For instance, if there is a function named `add` in the `math` module inside the `book` package, its full path would be `book::math::add`. Once the module has been imported, the function can be accessed directly as `math::add`, as shown in the following example:


```move
module book::usage;

use book::math::{Self};

public fun use_addition(): u64 {
    let sum = math::add(10, 20);
    sum
}
```

### Entry functions

Functions can be invoked from user transactions by declaring them as `entry`. The difference between `public` and `entry` is that public functions are accessible from other modules whereas can be called in transactions. A function can be both `public` and `entry` at the same time.

```move
module book::transaction_example;

entry fun perform_addition(a: u64, b: u64) {
    // Perform some state change
}
```

> [!NOTE]
> You must change the function name to `camelCase` when calling an entry function from a transaction. For example, the `perform_addition` function would be called as `performAddition` in a transaction.
>
> This is to follow Solidity's naming conventions.

## Multiple return values

Move functions are capable of returning multiple values, which is especially useful when more than one piece of data needs to be produced by a function. The return type is expressed as a tuple of types, and the returned result is given as a tuple of expressions.

```move
fun divide_and_remainder(a: u64, b: u64): (u64, u64) {
    let quotient = a / b;
    let remainder = a % b;

    (quotient, remainder)
}
```

When a function call returns a tuple, the result must be unpacked into variables using the `let (tuple)` syntax.

```move
let (q, r) = divide_and_remainder(10, 3);
assert_eq!(q, 3);
assert_eq!(r, 1);
```

If a declared value must be mutable, the `mut` keyword is written before the variable name.

```move
let (mut q, r) = divide_and_remainder(10, 3);
q = q + 1;
assert_eq!(q, 4);
assert_eq!(r, 1);
```

When certain arguments are not needed, they can be ignored by using the `_` symbol.

```move
let (_, r) = divide_and_remainder(10, 3);
assert_eq!(r, 1);
```



