# Comments

Comments provide a way to annotate or document code. They are ignored by the compiler and do not produce any compiled WASM output. Common uses include explaining logic, leaving notes for collaborators, temporarily disabling code, or generating documentation.

### Line Comments

A line comment begins with `//`. Everything following `//` on that line is ignored by the compiler:

```move
// This is a line comment
let x = 10; // The compiler ignores this note
```

### Block Comments

Block comments allow you to comment out one or more lines of code. They begin with `/*` and end with `*/`. Everything between these delimiters is ignored by the compiler. Block comments can span multiple lines, a single line, or even part of a line.

```move
/* This is a block comment
   spanning multiple lines */
let x = 10;

/* You can also use them on a single line */
let y = 20;

/* Or even inline */ let z = 30; /* ignored */
```

## Doc Comments

Documentation comments (`///`) are used to generate API documentation directly from source code. They resemble block comments but are placed immediately before the definition of the item they describe. The compiler interprets them as structured documentation rather than ignoring them entirely.

```move
/// Represents a simple item with a value
struct Item has copy, drop {
    value: u64,
}

/// Creates a new `Item` with the given value
public fun new_item(x: u64): Item {
    Item { value: x }
}
```

## Whitespace

In Move, whitespace characters —such as spaces, tabs, and newlines— do not affect program semantics. They are used solely to improve readability and code formatting, without altering the behavior of the program.

