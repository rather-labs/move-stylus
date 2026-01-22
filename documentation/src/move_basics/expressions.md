# Expressions

In programming languages, an *expression* is a unit of code that evaluates to a value. In Move, almost everything is an expression, with the sole exception of the `let` statement, which is a declaration rather than an expression.Expressions are sequenced using semicolons (`;`). If no expression follows a semicolon, the compiler automatically inserts the unit value `()`, which represents an empty expression.

## Literals

A *literal* is a fixed value written directly in source code. Literals are commonly used to initialize variables or pass constant values as arguments to functions.

### Types of Literals

- **Boolean values**
  `true`, `false`
- **Integer values**
  Examples: `0`, `1`, `123123`
- **Hexadecimal values**
  Numbers prefixed with `0x` represent integers in hexadecimal form.
  Examples: `0x0`, `0x1`, `0x123`
- **Byte vector values**
  Prefixed with `b`, representing a sequence of bytes.
  Example: `b"bytes_vector"`
- **Byte values**
  Hexadecimal literals prefixed with `x`, representing raw byte sequences.
  Example: `x"0A"`

#### Example

```move
let flag = true;                    // Boolean literal
let count = 123;                    // Integer literal
let hex_num = 0xFF;                 // Hexadecimal literal
let bytes = b"hello world";         // Byte vector literal
let raw_byte = x"0A";               // Byte vector literal
let vec_literal = vector[1, 2, 3];  // vector[] is a vector literal
```

## Operators

Operators are used to perform arithmetic, logical, and bitwise operations on values. Since these operations always produce values, they are considered **expressions**.

#### Example

```move
// Arithmetic expression
let sum = 1 + 2;        // 1 + 2 is an expression
let sum = (1 + 2);      // same expression with parentheses

// Logical expression
let is_true = true && false;      // true && false is an expression
let is_true = (true && false);    // same expression with parentheses
```

## Blocks

A block in Move is a sequence of statements and expressions enclosed in curly braces `{}`. The block itself is an **expression**, and its value is determined by the last expression inside the block.

> The final expression must **not** end with a semicolon, otherwise the block evaluates to the unit value `()`.

#### Example

```move
// Block returning the value of its last expression
let x = {
    let a = 10;
    let b = 20;
    a + b   // last expression, no semicolon
};          // x = 30

// Block returning unit ()
let y = {
    let a = 5;
    a + 2;
};          // y = ()
```

## Function Calls

A [function](./move_basics/functions.md) call is an **expression**. When invoked, it executes the function body and returns the value of the last expression in that body, provided the final expression does **not** end with a semicolon. If the last expression ends with a semicolon, the function returns the unit value `()`.

#### Example

```move
fun add(x: u64, y: u64): u64 {
    x + y
}

fun log_value(x: u64): () {
    x;
}

// Function calls
let result = add(2, 3);         // result = 5
let unit_val = log_value(10);   // unit_val = ()
```

## Control Flow Expressions

[Control flow](./move_basics/control_flow.md) expressions determine how execution proceeds within a program. In Move, they are also **expressions**, meaning they evaluate to a value. The value returned depends on the branch or path taken.

```move
// if is an expression, so it returns a value.
// If there are 2 branches, the types of the branches must match.
if (bool_expr) expr1 else expr2;

// while is an expression, but it returns `()`.
while (bool_expr) { expr; };

// loop is an expression, but returns `()`.
loop { expr; break };

// Example with break returning a value
let val = loop {
    let x = 5;
    break x * 2; // loop returns 10
};
```
