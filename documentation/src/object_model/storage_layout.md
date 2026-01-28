# Storage Layout

Just like in Solidity, understanding how data is stored in Move is crucial for optimizing smart contract performance and ensuring efficient resource management. The way data is stored in storage follows the same [encoding principles as in Solidity](https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html). There are some exceptions that are specific to Move, but the general concepts remain the same.

Almost every data type is encoded in a way that optimizes for space, packing multiple smaller data types into a single 32-byte storage slot when possible.

## Typehash

Every type has a unique typehash, which is a `u64` value derived from type's canonical representation. This typehash is used to identify the type of data stored in a particular storage slot. The typehash is *always* present and it occupies the first 8 bytes of the storage slot.

For example, consider the following struct:

```move
public struct MyStruct has key {
    id: UID,
    a: u8,
    b: u16,
}
```

The storage layout for an instance of `MyStruct` would look like this:

```
n [                    bbatttttttt]
```

Where:
- `n` is the slot number.
- `tttttttt` is the 8-byte typehash for `MyStruct`.
- `a` is the byte representing the field `a`.
- `bb` is the 2-byte representation of the field `b`.

The typehash is used internally by the runtime to ensure that the data being accessed matches the expected type, providing a layer of type safety and to check if an slot is empty.

## Data sizes

The following table summarizes the sizes of various data types in Move:

| Data Type        | Size (bytes) |
|------------------|--------------|
| `bool`           | 1            |
| `u8`             | 1            |
| `u16`            | 2            |
| `u32`            | 4            |
| `u64`            | 8            |
| `u128`           | 16           |
| `u256`           | 32           |
| `address`        | 20           |


### Structs

#### Non-storage structs

If the struct is not an storage object, the struct is interpreted as a tuple of its fields. The fields are laid out in the order they are defined in the struct, with smaller fields packed together to optimize space:


```move
public struct PackedStruct {
    w: u8,
    x: u16,
    y: bool,
    z: u32,
}

public struct MyStruct has key {
    id: UID,
    a: u8,
    b: u16,
    c: PackedStruct,
}
```
The storage layout for an instance of `MyStruct` would look like this:

```
n [             zzzzyxxwbbatttttttt]
                └───┬──┘
                    ▼
                PackedStruct
```

### Storage structs

The case of nested storage objects is different. Each storage object is stored in its own storage slot, and the parent struct only contains a reference (the `UID`) to the child storage object:

```move
public struct PackedStruct has key, store{
    id: UID,
    w: u8,
    x: u16,
    y: bool,
    z: u32,
}

public struct MyStruct has key {
    id: UID,
    a: u8,
    b: u16,
    c: PackedStruct,
}
```

The storage layout for an instance of `MyStruct` would look like this:

```
MyStruct:

n   [                     bbatttttttt]
n+1 [<bytes storing PackedStruct ID> ] (32 bytes)



PackedStruct
m   [                zzzzyxxwuuuuuuuu]
```

Where:
- `n` is the first slot of `MyStruct`.
- `n+1` is the second slot of `MyStruct`, which contains the `UID` of the `PackedStruct`.
- `tttttttt` is the 8-byte typehash for `MyStruct`.
- `a` is the byte representing the field `a`.
- `b` is the 2-byte representation of the field `b`.
- `m` is the slot where the actual `PackedStruct` data is stored, identified by the `UID` in slot `n+1`.
- `uuuuuuuu` is the 8-byte typehash for `PackedStruct`.
- `w` is the byte representing the field `w`.
- `x` is the 2-byte representation of the field `x`.
- `y` is the byte representing the field `y`.
- `z` is the 4-byte representation of the field `z`.

### Enums

#### Simple enums

If the enum is simple (i.e., none of the variants contain data), it is stored as a single byte representing the variant index.

```move
public enum SimpleEnum {
    A,
    B,
    C,
}

public struct MyStruct has key {
    id: UID,
    e: SimpleEnum,
}
```

The storage layout for an instance of `MyStruct` would look like this:

```
n [                       etttttttt]
```

Where:
- `n` is the slot number.
- `tttttttt` is the 8-byte typehash for `MyStruct`.
- `e` is the byte representing the variant index of `SimpleEnum`.

#### Complex enums

If the enum is complex (i.e., some variants contain data), it is stored with a discriminant byte followed by the data for the active variant.

> [!NOTE]
> Since complex enums can vary in size depending on the active variant, the space used by the enum is the size of the larger variant.

```move
public enum ComplexEnum {
    A(u8),
    B(u16, u32),
    C,
}

public struct MyStruct has key {
    id: UID,
    e: ComplexEnum,
}
```

The storage layout for an instance of `MyStruct` would look like this:

```
n [                 BBBBBBbtttttttt]
```

Where:
- `n` is the slot number.
- `tttttttt` is the 8-byte typehash for `MyStruct`.
- `b` is the byte representing the variant of `ComplexEnum`.
- `BBBBBB` is the space allocated for the largest variant of `ComplexEnum` (in this case, variant `B` which contains a `u16` and a `u32`

### Dynamic arryas and Strings

Like in Solidity, dynamically sized arrays have unpredictable sizes, so they cannot be placed between other state variables in storage. For layout purposes, they are treated as occupying a fixed 32 bytes, while their actual contents are stored separately, beginning at a storage slot determined by a Keccak‑256 hash.

To know more about how dynamic arrays and strings are stored, refer to the [Solidity documentation on dynamic arrays](https://docs.soliditylang.org/en/latest/internals/layout_in_storage.html#mappings-and-dynamic-arrays).

> [!NOTE]
> Strings in Move are implemented as `vector<u8>`, so they follow the same storage layout rules as dynamic arrays.

## Data ordering

Data is ordered in storage based on the order of declaration in the struct. This means that the order of the fields in the struct definition directly affects how they are laid out in storage. For example, consider the following struct:

```move
pub struct Inefficient has key{
    id: UID,
    a: u8,
    b: u256,
    c: u8,
    d: u256,
}
```

The storage layout for an instance of `Inefficient` would look like this:

```
n     [                       atttttttt]
n + 1 [bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb]
n + 2 [                               c]
n + 3 [dddddddddddddddddddddddddddddddd]
```

Occupying 4 storage slots.

To optimize the storage layout, we can reorder the fields to group smaller data types together:

```move
pub struct Efficient has key{
    id: UID,
    a: u8,
    c: u8,
    b: u256,
    d: u256,
}
```

The storage layout for an instance of `Efficient` would look like this:

```
n     [                      catttttttt]
n + 1 [bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb]
n + 2 [dddddddddddddddddddddddddddddddd]
```

> [!Important]
> All the things valid for `UID` are also valid for `NamedId` as well.
