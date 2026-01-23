# Control Flow

## Control Flow Statements

Control flow statements determine how a program executes by directing its path. They allow you to make decisions, repeat sections of code, or exit early from a block or function.

Move includes the following control flow statements:

- [**if / if-else**](#conditional-statements) — decide whether a block of code should run.
- [**loop / while**](#repeating-code-with-loops) — repeat a block of code until a condition is met.
- [**break / continue**](#exiting-loops-early) — exit a loop early or skip to the next iteration.
- [**return**](#early-return) — exit a function before reaching its end.

## Conditional Statements

The `if` expression allows you to execute a block of code only if a specified condition is true. You can also use `else` to provide an alternative block of code if the condition is false.

The syntax for an `if` expression is as follows:

```
if (<bool_expression) <expression>;
if (<bool_expression) <expression> else <expression>;
```

The `else` keyword is optional. If the condition evaluates to true, the first expression is executed; otherwise, the second expression (if provided) is executed. If the `else` clause is used, both branches must return values of the same type.

Like any other expression, `if` requires a semicolon at the end if there are other expressions following it.


Here are some examples of using `if` and `if-else` statements in Move:

```move
// Example of an if statement
let x = 10;
if (x > 5) {
    // This block executes because the condition is true
    let y = x * 2;
};

// Example of an if-else statement
let a = 3;
let b = 7;
let max;
let max = if (a > b) {
    a; // This block does not execute
} else {
    b; // This block executes because the condition is false
};
```

Conditional expressions are among the most important control flow statements in Move.  They evaluate user-provided input or stored data to make decisions. One key use case is the [`assert!`](./TODO.md) macro, which verifies that a condition is true and aborts execution if it is not.

## Repeating Code with Loops

Loops allow you to repeat a block of code multiple times based on a condition. Move supports two types of loops: `loop` and `while`. In many cases, you can use either type of loop to achieve the same result, but usuallly `while` loops are more concise when the number of iterations is determined by a condition while `loop` is more flexible for infinite loops or when the exit condition is complex.

### The `while` Loop

The `while` loop repeatedly executes a block of code as long as a specified condition evaluates to true.  The boolean expression is evaluated before each iteration, and if it evaluates to false, the loop terminates.

The syntax for a `while` loop is as follows:

```
while (<bool_expression>) { <expression>; };
```

Here is an example of using a `while` loop in Move:

```move
let mut count = 0;

while (count < 5) {
    // This block executes as long as count is less than 5
    count = count + 1;
};

assert_eq!(count, 5);
```

### Infinite Loops with `loop`

The `loop` statement creates an infinite loop that continues executing until it is explicitly exited using a `break` statement. This type of loop is useful when the number of iterations is not known in advance or when you want to create a loop that runs indefinitely until a certain condition is met.

The syntax for a `loop` statement is as follows:

```
loop { <expression>; };
```

Let's rewrite the previous `while` loop example using a `loop` statement:

```move
let mut count = 0;

loop {
    if (count >= 5) {
        break; // Exit the loop when count reaches 5
    };
    count = count + 1;
};

assert_eq!(count, 5);
```

If the if expression was not used inside the loop, the loop would run indefinitely, causing the program to hang or crash.

### Exiting Loops Early

You can exit a loop early using the `break` statement. The `break` statement immediately terminates the nearest enclosing loop and transfers control to the statement following the loop. It is usually used in conjunction with conditional statements to determine when to exit the loop (as seen in the previous example). It can be used in both `loop` and `while` loops.

### Skipping to the Next Iteration

The `continue` statement allows you to skip the current iteration of a loop and proceed to the next iteration. When `continue` is encountered, the remaining code in the loop body for that iteration is skipped, and the loop condition is re-evaluated (for `while` loops) or the next iteration begins (for `loop` statements). It is typically used within conditional statements to skip certain iterations based on specific criteria.

Here is an example of using the `continue` statement in a `while` loop:

```move
let mut count = 0;
let mut sum = 0;

while (count < 10) {
    count = count + 1;

    if (count % 2 == 0) {
        continue; // Skip even numbers
    };

    sum = sum + count; // Only odd numbers are added to sum
};
```

## Early Return

The `return` statement allows you to exit a function before reaching its end and optionally return a value to the caller. When `return` is executed, the function terminates immediately, and control is transferred back to the point where the function was called.

The syntax for the `return` statement is as follows:

```
return <expression>
```

Here is an example of using the `return` statement in a Move function:

```move
public fun is_odd(num: u64): bool {
    if (num % 2 == 1) {
        return true
    };
    false
}
```

Unlike in many other languages, the `return` statement is not required for the last expression in a function.
In Move, the final expression in a function block is automatically returned. However, the `return` statement is useful when you want to exit a function early if a certain condition is met.

