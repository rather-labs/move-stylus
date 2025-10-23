module 0x01::equality_enums;

// Simple enum with primitive types
public enum SimpleEnum has drop {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    U256(u256),
    Address(address),
}

// Enum with vector types
public enum VectorEnum has drop {
    StackVector(vector<u32>),
    HeapVector(vector<u128>),
    BoolVector(vector<bool>),
    AddressVector(vector<address>),
}

// Enum with struct fields
public struct InnerStruct has drop {
    x: u32,
    y: u128,
}

public enum StructEnum has drop {
    WithStruct(InnerStruct),
    WithPrimitives { a: u8, b: u16, c: u32 },
    Mixed { struct_field: InnerStruct, primitive: u64 },
}

// Complex enum with nested structures
public enum ComplexEnum has drop {
    Simple(u32),
    WithVector(vector<u64>),
    WithStruct(InnerStruct),
    Nested { inner: InnerStruct, vec: vector<u32>, flag: bool },
}

// Test equality for simple enum with bool
entry fun eq_simple_enum_bool(a: bool, b: bool): bool {
    let enum1 = SimpleEnum::Bool(a);
    let enum2 = SimpleEnum::Bool(b);
    enum1 == enum2
}

// Test equality for simple enum with u8
entry fun eq_simple_enum_u8(a: u8, b: u8): bool {
    let enum1 = SimpleEnum::U8(a);
    let enum2 = SimpleEnum::U8(b);
    enum1 == enum2
}

// Test equality for simple enum with u16
entry fun eq_simple_enum_u16(a: u16, b: u16): bool {
    let enum1 = SimpleEnum::U16(a);
    let enum2 = SimpleEnum::U16(b);
    enum1 == enum2
}

// Test equality for simple enum with u32
entry fun eq_simple_enum_u32(a: u32, b: u32): bool {
    let enum1 = SimpleEnum::U32(a);
    let enum2 = SimpleEnum::U32(b);
    enum1 == enum2
}

// Test equality for simple enum with u64
entry fun eq_simple_enum_u64(a: u64, b: u64): bool {
    let enum1 = SimpleEnum::U64(a);
    let enum2 = SimpleEnum::U64(b);
    enum1 == enum2
}

// Test equality for simple enum with u128
entry fun eq_simple_enum_u128(a: u128, b: u128): bool {
    let enum1 = SimpleEnum::U128(a);
    let enum2 = SimpleEnum::U128(b);
    enum1 == enum2
}

// Test equality for simple enum with u256
entry fun eq_simple_enum_u256(a: u256, b: u256): bool {
    let enum1 = SimpleEnum::U256(a);
    let enum2 = SimpleEnum::U256(b);
    enum1 == enum2
}

// Test equality for simple enum with address
entry fun eq_simple_enum_address(a: address, b: address): bool {
    let enum1 = SimpleEnum::Address(a);
    let enum2 = SimpleEnum::Address(b);
    enum1 == enum2
}

// Test equality for vector enum with stack type
entry fun eq_vector_enum_stack(a: vector<u32>, b: vector<u32>): bool {
    let enum1 = VectorEnum::StackVector(a);
    let enum2 = VectorEnum::StackVector(b);
    enum1 == enum2
}

// Test equality for vector enum with heap type
entry fun eq_vector_enum_heap(a: vector<u128>, b: vector<u128>): bool {
    let enum1 = VectorEnum::HeapVector(a);
    let enum2 = VectorEnum::HeapVector(b);
    enum1 == enum2
}

// Test equality for vector enum with bool vector
entry fun eq_vector_enum_bool(a: vector<bool>, b: vector<bool>): bool {
    let enum1 = VectorEnum::BoolVector(a);
    let enum2 = VectorEnum::BoolVector(b);
    enum1 == enum2
}

// Test equality for vector enum with address vector
entry fun eq_vector_enum_address(a: vector<address>, b: vector<address>): bool {
    let enum1 = VectorEnum::AddressVector(a);
    let enum2 = VectorEnum::AddressVector(b);
    enum1 == enum2
}

// Test equality for struct enum with struct field
entry fun eq_struct_enum_with_struct(a: u32, b: u128, c: u32, d: u128): bool {
    let struct1 = InnerStruct { x: a, y: b };
    let struct2 = InnerStruct { x: c, y: d };
    let enum1 = StructEnum::WithStruct(struct1);
    let enum2 = StructEnum::WithStruct(struct2);
    enum1 == enum2
}

// Test equality for struct enum with primitive fields
entry fun eq_struct_enum_with_primitives(a: u8, b: u16, c: u32, d: u8, e: u16, f: u32): bool {
    let enum1 = StructEnum::WithPrimitives { a, b, c };
    let enum2 = StructEnum::WithPrimitives { a: d, b: e, c: f };
    enum1 == enum2
}

// Test equality for struct enum with mixed fields
entry fun eq_struct_enum_mixed(a: u32, b: u128, c: u64, d: u32, e: u128, f: u64): bool {
    let struct1 = InnerStruct { x: a, y: b };
    let struct2 = InnerStruct { x: d, y: e };
    let enum1 = StructEnum::Mixed { struct_field: struct1, primitive: c };
    let enum2 = StructEnum::Mixed { struct_field: struct2, primitive: f };
    enum1 == enum2
}

// Test equality for complex enum with simple variant
entry fun eq_complex_enum_simple(a: u32, b: u32): bool {
    let enum1 = ComplexEnum::Simple(a);
    let enum2 = ComplexEnum::Simple(b);
    enum1 == enum2
}

// Test equality for complex enum with vector variant
entry fun eq_complex_enum_vector(a: vector<u64>, b: vector<u64>): bool {
    let enum1 = ComplexEnum::WithVector(a);
    let enum2 = ComplexEnum::WithVector(b);
    enum1 == enum2
}

// Test equality for complex enum with struct variant
entry fun eq_complex_enum_struct(a: u32, b: u128, c: u32, d: u128): bool {
    let struct1 = InnerStruct { x: a, y: b };
    let struct2 = InnerStruct { x: c, y: d };
    let enum1 = ComplexEnum::WithStruct(struct1);
    let enum2 = ComplexEnum::WithStruct(struct2);
    enum1 == enum2
}

// Test equality for complex enum with nested variant
entry fun eq_complex_enum_nested(a: u32, b: u128, c: vector<u32>, d: bool, e: u32, f: u128, g: vector<u32>, h: bool): bool {
    let struct1 = InnerStruct { x: a, y: b };
    let struct2 = InnerStruct { x: e, y: f };
    let enum1 = ComplexEnum::Nested { inner: struct1, vec: c, flag: d };
    let enum2 = ComplexEnum::Nested { inner: struct2, vec: g, flag: h };
    enum1 == enum2
}

// Test inequality for simple enum with bool
entry fun neq_simple_enum_bool(a: bool, b: bool): bool {
    let enum1 = SimpleEnum::Bool(a);
    let enum2 = SimpleEnum::Bool(b);
    enum1 != enum2
}

// Test inequality for simple enum with u8
entry fun neq_simple_enum_u8(a: u8, b: u8): bool {
    let enum1 = SimpleEnum::U8(a);
    let enum2 = SimpleEnum::U8(b);
    enum1 != enum2
}

// Test inequality for simple enum with u16
entry fun neq_simple_enum_u16(a: u16, b: u16): bool {
    let enum1 = SimpleEnum::U16(a);
    let enum2 = SimpleEnum::U16(b);
    enum1 != enum2
}

// Test inequality for simple enum with u32
entry fun neq_simple_enum_u32(a: u32, b: u32): bool {
    let enum1 = SimpleEnum::U32(a);
    let enum2 = SimpleEnum::U32(b);
    enum1 != enum2
}

// Test inequality for simple enum with u64
entry fun neq_simple_enum_u64(a: u64, b: u64): bool {
    let enum1 = SimpleEnum::U64(a);
    let enum2 = SimpleEnum::U64(b);
    enum1 != enum2
}

// Test inequality for simple enum with u128
entry fun neq_simple_enum_u128(a: u128, b: u128): bool {
    let enum1 = SimpleEnum::U128(a);
    let enum2 = SimpleEnum::U128(b);
    enum1 != enum2
}

// Test inequality for simple enum with u256
entry fun neq_simple_enum_u256(a: u256, b: u256): bool {
    let enum1 = SimpleEnum::U256(a);
    let enum2 = SimpleEnum::U256(b);
    enum1 != enum2
}

// Test inequality for simple enum with address
entry fun neq_simple_enum_address(a: address, b: address): bool {
    let enum1 = SimpleEnum::Address(a);
    let enum2 = SimpleEnum::Address(b);
    enum1 != enum2
}

// Test inequality for vector enum with stack type
entry fun neq_vector_enum_stack(a: vector<u32>, b: vector<u32>): bool {
    let enum1 = VectorEnum::StackVector(a);
    let enum2 = VectorEnum::StackVector(b);
    enum1 != enum2
}

// Test inequality for vector enum with heap type
entry fun neq_vector_enum_heap(a: vector<u128>, b: vector<u128>): bool {
    let enum1 = VectorEnum::HeapVector(a);
    let enum2 = VectorEnum::HeapVector(b);
    enum1 != enum2
}

// Test inequality for vector enum with bool vector
entry fun neq_vector_enum_bool(a: vector<bool>, b: vector<bool>): bool {
    let enum1 = VectorEnum::BoolVector(a);
    let enum2 = VectorEnum::BoolVector(b);
    enum1 != enum2
}

// Test inequality for vector enum with address vector
entry fun neq_vector_enum_address(a: vector<address>, b: vector<address>): bool {
    let enum1 = VectorEnum::AddressVector(a);
    let enum2 = VectorEnum::AddressVector(b);
    enum1 != enum2
}

// Test inequality for struct enum with struct field
entry fun neq_struct_enum_with_struct(a: u32, b: u128, c: u32, d: u128): bool {
    let struct1 = InnerStruct { x: a, y: b };
    let struct2 = InnerStruct { x: c, y: d };
    let enum1 = StructEnum::WithStruct(struct1);
    let enum2 = StructEnum::WithStruct(struct2);
    enum1 != enum2
}

// Test inequality for struct enum with primitive fields
entry fun neq_struct_enum_with_primitives(a: u8, b: u16, c: u32, d: u8, e: u16, f: u32): bool {
    let enum1 = StructEnum::WithPrimitives { a, b, c };
    let enum2 = StructEnum::WithPrimitives { a: d, b: e, c: f };
    enum1 != enum2
}

// Test inequality for struct enum with mixed fields
entry fun neq_struct_enum_mixed(a: u32, b: u128, c: u64, d: u32, e: u128, f: u64): bool {
    let struct1 = InnerStruct { x: a, y: b };
    let struct2 = InnerStruct { x: d, y: e };
    let enum1 = StructEnum::Mixed { struct_field: struct1, primitive: c };
    let enum2 = StructEnum::Mixed { struct_field: struct2, primitive: f };
    enum1 != enum2
}

// Test inequality for complex enum with simple variant
entry fun neq_complex_enum_simple(a: u32, b: u32): bool {
    let enum1 = ComplexEnum::Simple(a);
    let enum2 = ComplexEnum::Simple(b);
    enum1 != enum2
}

// Test inequality for complex enum with vector variant
entry fun neq_complex_enum_vector(a: vector<u64>, b: vector<u64>): bool {
    let enum1 = ComplexEnum::WithVector(a);
    let enum2 = ComplexEnum::WithVector(b);
    enum1 != enum2
}

// Test inequality for complex enum with struct variant
entry fun neq_complex_enum_struct(a: u32, b: u128, c: u32, d: u128): bool {
    let struct1 = InnerStruct { x: a, y: b };
    let struct2 = InnerStruct { x: c, y: d };
    let enum1 = ComplexEnum::WithStruct(struct1);
    let enum2 = ComplexEnum::WithStruct(struct2);
    enum1 != enum2
}

// Test inequality for complex enum with nested variant
entry fun neq_complex_enum_nested(a: u32, b: u128, c: vector<u32>, d: bool, e: u32, f: u128, g: vector<u32>, h: bool): bool {
    let struct1 = InnerStruct { x: a, y: b };
    let struct2 = InnerStruct { x: e, y: f };
    let enum1 = ComplexEnum::Nested { inner: struct1, vec: c, flag: d };
    let enum2 = ComplexEnum::Nested { inner: struct2, vec: g, flag: h };
    enum1 != enum2
}

// Test equality between different variants of the same enum (should always be false)
entry fun eq_different_variants_simple(a: u32, b: u64): bool {
    let enum1 = SimpleEnum::U32(a);
    let enum2 = SimpleEnum::U64(b);
    enum1 == enum2
}

// Test equality between different variants of vector enum
entry fun eq_different_variants_vector(a: vector<u32>, b: vector<u128>): bool {
    let enum1 = VectorEnum::StackVector(a);
    let enum2 = VectorEnum::HeapVector(b);
    enum1 == enum2
}

// Test equality between different variants of struct enum
entry fun eq_different_variants_struct(a: u32, b: u128, c: u8, d: u16, e: u32): bool {
    let struct_field = InnerStruct { x: a, y: b };
    let enum1 = StructEnum::WithStruct(struct_field);
    let enum2 = StructEnum::WithPrimitives { a: c, b: d, c: e };
    enum1 == enum2
}

// Test equality between different variants of complex enum
entry fun eq_different_variants_complex(a: u32, b: vector<u64>): bool {
    let enum1 = ComplexEnum::Simple(a);
    let enum2 = ComplexEnum::WithVector(b);
    enum1 == enum2
}

// Test inequality between different variants of the same enum (should always be true)
entry fun neq_different_variants_simple(a: u32, b: u64): bool {
    let enum1 = SimpleEnum::U32(a);
    let enum2 = SimpleEnum::U64(b);
    enum1 != enum2
}

// Test inequality between different variants of vector enum
entry fun neq_different_variants_vector(a: vector<u32>, b: vector<u128>): bool {
    let enum1 = VectorEnum::StackVector(a);
    let enum2 = VectorEnum::HeapVector(b);
    enum1 != enum2
}

// Test inequality between different variants of struct enum
entry fun neq_different_variants_struct(a: u32, b: u128, c: u8, d: u16, e: u32): bool {
    let struct_field = InnerStruct { x: a, y: b };
    let enum1 = StructEnum::WithStruct(struct_field);
    let enum2 = StructEnum::WithPrimitives { a: c, b: d, c: e };
    enum1 != enum2
}

// Test inequality between different variants of complex enum
entry fun neq_different_variants_complex(a: u32, b: vector<u64>): bool {
    let enum1 = ComplexEnum::Simple(a);
    let enum2 = ComplexEnum::WithVector(b);
    enum1 != enum2
}

// Test equality for vector of simple enums
entry fun eq_vector_simple_enums(a: vector<u8>, b: vector<u8>): bool {
    if (vector::length(&a) != vector::length(&b)) {
        return false
    };

    let mut vec1 = vector::empty<SimpleEnum>();
    let mut vec2 = vector::empty<SimpleEnum>();
    
    let mut i = 0;
    while (i < vector::length(&a)) {
        vector::push_back(&mut vec1, SimpleEnum::U8(*vector::borrow(&a, i)));
        vector::push_back(&mut vec2, SimpleEnum::U8(*vector::borrow(&b, i)));
        i = i + 1;
    };
    
    vec1 == vec2
}

// Test inequality for vector of simple enums
entry fun neq_vector_simple_enums(a: vector<u8>, b: vector<u8>): bool {
    if (vector::length(&a) != vector::length(&b)) {
        return true
    };

    let mut vec1 = vector::empty<SimpleEnum>();
    let mut vec2 = vector::empty<SimpleEnum>();
    
    let mut i = 0;
    while (i < vector::length(&a)) {
        vector::push_back(&mut vec1, SimpleEnum::U8(*vector::borrow(&a, i)));
        vector::push_back(&mut vec2, SimpleEnum::U8(*vector::borrow(&b, i)));
        i = i + 1;
    };
    
    vec1 != vec2
}

// Test equality for vector of struct enums
entry fun eq_vector_struct_enums(a: vector<u32>, b: vector<u128>, c: vector<u32>, d: vector<u128>): bool {
    if (vector::length(&a) != vector::length(&c)) {
        return false
    };

    let mut vec1 = vector::empty<StructEnum>();
    let mut vec2 = vector::empty<StructEnum>();
    
    let mut i = 0;
    while (i < vector::length(&a)) {
        let struct1 = InnerStruct { x: *vector::borrow(&a, i), y: *vector::borrow(&b, i) };
        let struct2 = InnerStruct { x: *vector::borrow(&c, i), y: *vector::borrow(&d, i) };
        vector::push_back(&mut vec1, StructEnum::WithStruct(struct1));
        vector::push_back(&mut vec2, StructEnum::WithStruct(struct2));
        i = i + 1;
    };
    
    vec1 == vec2
}

// Test inequality for vector of struct enums
entry fun neq_vector_struct_enums(a: vector<u32>, b: vector<u128>, c: vector<u32>, d: vector<u128>): bool {
    if (vector::length(&a) != vector::length(&c)) {
        return true
    };

    let mut vec1 = vector::empty<StructEnum>();
    let mut vec2 = vector::empty<StructEnum>();
    
    let mut i = 0;
    while (i < vector::length(&a)) {
        let struct1 = InnerStruct { x: *vector::borrow(&a, i), y: *vector::borrow(&b, i) };
        let struct2 = InnerStruct { x: *vector::borrow(&c, i), y: *vector::borrow(&d, i) };
        vector::push_back(&mut vec1, StructEnum::WithStruct(struct1));
        vector::push_back(&mut vec2, StructEnum::WithStruct(struct2));
        i = i + 1;
    };
    
    vec1 != vec2
}

// Test equality for vector of complex enums
entry fun eq_vector_complex_enums(a: vector<u32>, b: vector<u32>): bool {
    if (vector::length(&a) != vector::length(&b)) {
        return false
    };

    let mut vec1 = vector::empty<ComplexEnum>();
    let mut vec2 = vector::empty<ComplexEnum>();
    
    let mut i = 0;
    while (i < vector::length(&a)) {
        vector::push_back(&mut vec1, ComplexEnum::Simple(*vector::borrow(&a, i)));
        vector::push_back(&mut vec2, ComplexEnum::Simple(*vector::borrow(&b, i)));
        i = i + 1;
    };
    
    vec1 == vec2
}

// Test inequality for vector of complex enums
entry fun neq_vector_complex_enums(a: vector<u32>, b: vector<u32>): bool {
    if (vector::length(&a) != vector::length(&b)) {
        return true
    };

    let mut vec1 = vector::empty<ComplexEnum>();
    let mut vec2 = vector::empty<ComplexEnum>();
    
    let mut i = 0;
    while (i < vector::length(&a)) {
        vector::push_back(&mut vec1, ComplexEnum::Simple(*vector::borrow(&a, i)));
        vector::push_back(&mut vec2, ComplexEnum::Simple(*vector::borrow(&b, i)));
        i = i + 1;
    };
    
    vec1 != vec2
}

// Test equality for vector of mixed enum variants
entry fun eq_vector_mixed_enums(a: vector<u32>, b: vector<u64>): bool {
    if (vector::length(&a) != vector::length(&b)) {
        return false
    };

    let mut vec1 = vector::empty<SimpleEnum>();
    let mut vec2 = vector::empty<SimpleEnum>();
    
    let mut i = 0;
    while (i < vector::length(&a)) {
        vector::push_back(&mut vec1, SimpleEnum::U32(*vector::borrow(&a, i)));
        vector::push_back(&mut vec2, SimpleEnum::U64(*vector::borrow(&b, i)));
        i = i + 1;
    };
    
    vec1 == vec2
}

// Test inequality for vector of mixed enum variants
entry fun neq_vector_mixed_enums(a: vector<u32>, b: vector<u64>): bool {
    if (vector::length(&a) != vector::length(&b)) {
        return true
    };

    let mut vec1 = vector::empty<SimpleEnum>();
    let mut vec2 = vector::empty<SimpleEnum>();
    
    let mut i = 0;
    while (i < vector::length(&a)) {
        vector::push_back(&mut vec1, SimpleEnum::U32(*vector::borrow(&a, i)));
        vector::push_back(&mut vec2, SimpleEnum::U64(*vector::borrow(&b, i)));
        i = i + 1;
    };
    
    vec1 != vec2
}

