# Primitive Types

Move provides a set of built-in primitive types for representing simple values. These types form the foundation upon which all other types are constructed. The primary primitive types are:

- [**Booleans**](#booleans)
- [**Unsigned integers**](#unsigned-integers)
- **Addresses** (covered in the next section)

Before exploring each primitive type in detail, it is useful to understand how variables are declared and assigned in Move.

## Variables and Assignment

Variables are declared using the `let` keyword. By default, variables are **immutable**, meaning their values cannot be changed after initialization. To declare a mutable variable, the `mut` keyword must be added before the variable name.

```
let <variable_name>[: <type>]  = <expression>;
let mut <variable_name>[: <type>] = <expression>;
```

Where:
- `<variable_name>` is the name of the variable being declared.
- `<type>` is an optional type annotation specifying the variable's type.
- `<expression>` is the value to be assigned to the variable.

### Example
```move
// Immutable variable
let x: u64 = 10;

// Mutable variable
let mut y: u64 = 20;
y = y + 1; // allowed because y is mutable
```

## Booleans

The `bool` type represents a boolean value, which can be either `true` or `false`. These are reserved keywords in Move. Since the compiler can infer the type directly from the literal value, it is not necessary to explicitly annotate booleans with their type.

```move
let flag = true;  // type inferred as bool
let is_valid = false;
```

## Unsigned Integers

Move provides a set of unsigned integer types with fixed bit widths, ranging from 8 bits to 256 bits. These types are used to represent non-negative integer values and differ in the maximum value they can store.

The available integer types are:

- `u8`   — 8-bit unsigned integer
- `u16`  — 16-bit unsigned integer
- `u32`  — 32-bit unsigned integer
- `u64`  — 64-bit unsigned integer
- `u128` — 128-bit unsigned integer
- `u256` — 256-bit unsigned integer

#### Example

```move
let small: u8 = 255;        // maximum value for u8
let medium: u64 = 1_000;    // u64 can hold larger values
let large: u256 = 1_000_000_000; // u256 supports very large integers
```

### Integer Literals and Type Inference

Boolean literals such as `true` and `false` are unambiguous and always represent values of type `bool`. In contrast, integer literals (e.g., `42`) can correspond to any of the available unsigned integer types.

In most cases, the compiler infers the type automatically, defaulting to `u64` when no additional context is provided. However, there are situations where type inference is insufficient, and an explicit type annotation is required. This can be achieved in two ways:

1. Type annotation during assignment
2. Type suffix applied directly to the literal

#### Examples

```move
// Compiler infers type as u64
let a = 42;

// Explicit type annotation
let b: u8 = 42;

// Type suffix
let c = 42u8;
let d = 1000u128;
```

### Arithmetic Operations

Move supports the standard arithmetic operations for unsigned integers: addition, subtraction, multiplication, division, and modulus (remainder). Each operation has well-defined semantics and may abort under specific conditions.

| Syntax | Operation       | Aborts If                                      |
|--------|-----------------|------------------------------------------------|
| `+`    | Addition        | Result exceeds the maximum value of the type   |
| `-`    | Subtraction     | Result is less than zero                       |
| `*`    | Multiplication  | Result exceeds the maximum value of the type   |
| `%`    | Modulus         | Divisor is `0`                                 |
| `/`    | Division        | Divisor is `0`                                 |


#### Example

```move
let a: u64 = 10;
let b: u64 = 3;

let sum = a + b;        // 13
let diff = a - b;       // 7
let product = a * b;    // 30
let quotient = a / b;   // 3 (truncating division)
let remainder = a % b;  // 1
```

### Bitwise Operations

Integer types in support bitwise operations, which treat values as sequences of bits (`0` or `1`) rather than numerical integers. Bitwise operations do not abort.

| Syntax | Operation     | Description                                      |
|--------|---------------|--------------------------------------------------|
| `&`    | Bitwise AND   | Performs a boolean AND on each bit pairwise       |
| `\|`    | Bitwise OR    | Performs a boolean OR on each bit pairwise        |
| `^`    | Bitwise XOR   | Performs a boolean exclusive OR on each bit pairwise |

#### Example

```move
let a: u8 = 0b1010;
let b: u8 = 0b1100;

let and_result = a & b; // 0b1000
let or_result  = a | b; // 0b1110
let xor_result = a ^ b; // 0b0110
```

### Bit Shifts

Each integer type supports bit shifts. The right-hand side operand (the number of bits to shift) must always be a u8.

Bit shifts can abort if the shift amount is greater than or equal to the bit width of the type (8, 16, 32, 64, 128, or 256).

| Syntax | Operation     | Aborts if                               |
|--------|---------------|-----------------------------------------|
| `>>`    | Shift Right  | Shift amount ≥ size of the integer type |
| `<<`    | Shift Left   | Shift amount ≥ size of the integer type |


#### Example

```move
let a: u8 = 0b0001_0000;
let left_shift = a << 2;  // 0b0100_0000
let right_shift = a >> 2; // 0b0000_0100
```

### Comparisons

Integer types are the only types in Move that support comparison operators. Both operands must be of the same type; otherwise, explicit casting is required. Comparison operations do not abort.

| Syntax | Operation                |
| ------ | ------------------------ |
| `<`    | less than                |
| `>`    | greater than             |
| `<=`   | less than or equal to    |
| `>=`   | greater than or equal to |

#### Example

```move
let a: u64 = 10;
let b: u64 = 20;

let is_less = a < b;   // true
let is_equal = a == b; // false
```

### Equality

All integer types support equality (`==`) and inequality (`!=`) operations. Both operands must be of the same type; otherwise, explicit casting is required. Equality operations do not abort.

| Syntax | Operation |
| ------ | --------- |
| `==`   | equal     |
| `!=`   | not equal |

#### Example

```move
let x: u16 = 42;
let y: u16 = 42;
let z: u16 = 7;

let eq = x == y; // true
let ne = x != z; // true
```

### Casting

Move supports explicit casting between integer types using the `as` keyword. This allows values of one integer type to be converted into another.

```
<expression> as <type>
```

Where:
- `<expression>` is the value to be cast.
- `<type>` is the target integer type.



Parentheses may be required to avoid ambiguity when casting within larger expressions.

#### Example

```move
// Basic casting
let x: u8 = 42;
let y: u16 = x as u16;

// Casting inside an expression (requires parentheses)
let z = 2 * (x as u16); // parentheses prevent ambiguity
```

Move does not permit silent overflow or underflow in arithmetic operations. If an operation produces a value outside the valid range of the type, the program will raise a **runtime error**. This behavior is a deliberate safety feature designed to prevent unexpected results and ensure that integer arithmetic remains predictable and secure.

