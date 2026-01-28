# Dynamic Fields

The Dynamic Fields API makes it possible to attach objects to other objects. Its behavior resembles that of a `Map` in other programming languages. Dynamic fields allow objects of any type to be attached.

There is no restriction on the number of dynamic fields that can be linked to an object.

## Definition

The Dynamic Fields API is defined in the `stylus::dynamic_fields` module. They are attached to object's `UID` or `NamedId` via a name, and can be accessed using that name. There can be only one field with a given name attached to an object.

The definition of the Dynamic Field is as follows:

```move
/// Internal object used for storing the field and value
public struct Field<Name: copy + drop + store, Value: store> has key {
    /// Determined by the hash of the object ID, the field name value and it's type,
    /// i.e. hash(parent.id || name || Name)
    id: UID,
    /// The value for the name of this field
    name: Name,
    /// The value bound to this field
    value: Value,
}
```

As defined, dynamic fields are maintained within an internal `Field` object. Its `UID` is generated deterministically from the object ID, the field name, and the field type. The `Field` object stores both the field name and the associated value. The constraints on the `Name` and `Value` type parameters specify the abilities required for the key and value.

> [!NOTE]
> Dynamic fields defined for `NamedId`s implemented in the `stylus::dynamic_fields_named_id` module have the same structure, they just replace `UID` for `NamedId`.


#### Example

```move
module book::dynamic_fields;

use stylus::dynamic_fields::{Self};

/// An example struct to be used as a dynamic field.
public struct Foo has key {
    id: object::UID,
}

/// Creates a new `Foo` object and shares it.
entry fun create_foo(ctx: &mut TxContext) {
    let foo = Foo { id: object::new(ctx) };
    transfer::share_object(foo);
}

/// Attaches a dynamic field to the `Foo` object
/// The key is the `name` parameter of type `String`
/// The value is the `value` parameter of type `u64`.
entry fun attach_dynamic_field(foo: &mut Foo, name: String, value: u64) {
    dynamic_field::add(&mut foo.id, name, value);
}
```

In this example, we define a struct `Foo` with a `UID`. We create an instance of `Foo` and share it. Then, we attach a dynamic field to the `Foo` object using the `dynamic_field::add` function, where the key is a `String` name and the value is a `u64`.


## Usage

The methods provided for dynamic fields are simple: a field can be added using `add`, removed with `remove`, and accessed through `borrow` or `borrow_mut`. In addition, the `exists_` method can be used to verify whether a field is present. For stricter type checks, the `exists_with_type` method is available.

Following the previous example, here are some additional functions that demonstrate how to read, check existence, mutate, and remove dynamic fields:

```move
/// Reads a dynamic field from the `Foo` object by its name.
entry fun read_dynamic_field(foo: &Foo, name: String): &u64 {
    dynamic_field::borrow(&foo.id, name)
}

/// Checks if a dynamic field exists in the `Foo` object by its name.
entry fun dynamic_field_exists(foo: &Foo, name: String): bool {
    dynamic_field::exists_(&foo.id, name)
}

/// Mutates a dynamic field in the `Foo` object by its name.
entry fun mutate_dynamic_field(foo: &mut Foo, name: String) {
    let val = dynamic_field::borrow_mut(&mut foo.id, name);
    *val = *val + 1;
}

/// Removes a dynamic field from the `Foo` object by its name.
entry fun remove_dynamic_field(foo: &mut Foo, name: String): u64 {
    let value = dynamic_field::remove(&mut foo.id, name);
    value
}
```

In this example, we define functions to read, check existence, mutate, and remove dynamic fields from the `Foo` object using the provided methods from the `stylus::dynamic_fields` module.

## Orphaned Dynamic Fields

The `object::delete()` function, which deletes an object with the provided `UID` from storage, does not track dynamic fields and therefore cannot prevent them from becoming orphaned. When the parent is deleted, its dynamic fields are not automatically removed. As a result, these dynamic fields remain stored but can no longer be accessed.


## Custom Type as a Field Name

In the previous examples, primitive types were used as field names because they possess the necessary abilities. Dynamic fields become even more powerful when custom types are employed as field names. This approach provides a more structured method of organizing data and also helps safeguard field names from being accessed by external modules.

```move
/// A custom type that includes fields.
public struct AccessoryKey has copy, drop, store { name: String }

/// An empty key, which can only be attached once.
public struct MetadataKey has copy, drop, store {}
```

The two field names defined earlier are `AccessoryKey` and `MetadataKey`. The `AccessoryKey` includes a `String` field, which allows it to be used multiple times with different `name` values. In contrast, the `MetadataKey` is an empty key and can only be attached once.

## Exposing `UID`

Granting mutable access to a `UID` or `NamedId` poses a security risk. Allowing a `UID` or `NamedId` to be exposed as a mutable reference can result in unintended modifications or even the removal of an object’s dynamic fields.  Therefore, it is essential to fully understand the consequences before exposing a `UID` as mutable.

Since dynamic fields are bound to `UID` or `NamedId` their usage in other modules depends on whether the `UID` or `NamedId` is accessible. By default, struct visibility safeguards the `id` field, preventing direct access from other modules. However, if a public accessor method returns a reference to the `UID` or `NamedId`, dynamic fields can be read externally.

```move
/// Exposes the UID of the Foo struct, so that other modules can read
/// dynamic fields.
public fun uid(f: &Foo): &UID {
    &f.id
}
```

In the example above, the `UID` of a `Foo` object is exposed. While this approach may be suitable for certain applications, it is important to keep in mind that exposing the `UID` permits access to *any* dynamic field attached to the object.

If `UID` exposure is required only within the package, consider using restrictive visibility such as `public(package)`. An even safer option is to provide specific accessor methods that allow reading only designated fields.

```move
/// Restrict UID access to modules within the same package.
public(package) fun uid_package(f: &Foo): &UID {
    &f.id
}

/// Provide access to borrow dynamic fields from a character.
public fun borrow<Name: copy + store + drop, Value: store>(
    c: &Character,
    n: Name
): &Value {
    stylus::dynamic_field::borrow(&c.id, n)
}
```

## Dynamic Fields vs. Regular Fields

Dynamic fields are more costly than regular fields because they demand extra storage and incur higher access costs. While they offer greater flexibility, this comes at a price. It is therefore important to weigh the trade-offs carefully when deciding between dynamic fields and regular fields.

