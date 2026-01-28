# Structs

Move's type system is particularly powerful when defining custom types. A struct lets developers model domain‑specific data structures that encapsulate both state and behavior. This makes it possible to design types that align closely with application requirements, beyond primitive values.

A custom type is declared using the `struct` keyword followed by the type name. The body of the struct contains its fields, each written in the form `field_name: field_type`. Fields must be separated by commas, and they can be of any type, including primitives, generics, or other structs. This flexibility allows developers to compose complex data models that reflect the domain requirements of their application.

> [!NOTE]
> Move does not support recursive structs, meaning a struct cannot contain itself as a field.

```move
/// A struct representing an author.
public struct Author {
    /// The name of the author.
    name: String,
}

/// A struct representing a book.
public struct Book {
    /// The title of the book.
    title: String,
    /// The author of the book. Uses the `Author` type.
    author: Author,
    /// The year the book was published.
    year: u16,
    /// Whether the book is the author’s first publication.
    is_first: bool,
    /// The edition number of the book, if any.
    edition: Option<u16>,
}
```
In this example, we define a Book struct with five fields. The `title` field is of type `String`, the `author` field is of type `Author`, the `year` field is of type `u16`, the `is_first` field is of type `bool`, and the `edition` field is of type `Option<u16>`. The `edition` field is optional, allowing you to represent books that may not have a specific edition number.

By default, the fields of a struct are private to the module in which the struct is defined. Direct field access from other modules is not allowed. To enable controlled access, the defining module must expose public functions that read or modify the fields.

## Creating and Using an Instance

Once a struct has been defined, it can be instantiated using the syntax: `StructName { field1: value1, field2: value2, ... }`. The order of fields in the initializer does not matter, but every field must be provided.


```move
// Creating an instance of the Author struct
let author = Author { name: b"Jane Doe".to_string() };
```

In the example above, we create an instance of the `Author` struct by providing a value for the `name` field. To access the fields of a struct, you can use the `.` operator.

```move
// Accessing the name field of the Author struct
let author_name = author.name;
```

Only the module that defines the struct can directly access its fields (both mutably and immutably). Other modules must use public functions provided by the defining module to read or modify the fields.

## Unpacking a struct

Struct values are non‑discardable by default. This means that once a struct is initialized, it must be used—either stored or unpacked into its constituent fields. Unpacking refers to deconstructing a struct into its fields so they can be accessed directly. The syntax uses the `let` keyword, followed by the struct name and the field names to bind each field to a local variable.

```move
// Unpacking the Author struct
let Author { name } = author;
```

In this example, we unpack the `author` instance of the `Author` struct, binding the `name` field to a local variable called `name`. After unpacking, you can use the `name` variable directly in your code. Because the value is not used, the compiler will raise a warning. To avoid this, you can use the underscore `_` to indicate that the variable is intentionally unused.

```move
// Unpacking the Author struct and ignoring unused fields
let Author { name: _ } = author;
```

## Struct with unnamed fields

Move also supports structs with unnamed fields, often referred to as tuple structs. These structs are defined similarly to regular structs but use parentheses instead of curly braces to enclose the fields. Each field is accessed by its index rather than by name.

```move
/// A struct representing a point in 2D space.
public struct Point(u64, u64);
```

In this example, we define a `Point` struct with two unnamed fields representing the x and y coordinates. To create an instance of this struct, you would use the following syntax:

```move
// Creating an instance of the Point struct
let point = Point(10, 20);
```

To access the fields of a tuple struct, you use the dot `.` operator followed by the index of the field (starting from 0).

```move
// Accessing the fields of the Point struct
let x = point.0;
let y = point.1;
```

> [!NOTE]
> In tuple structs, the order of fields matters, as they are accessed by their index.
