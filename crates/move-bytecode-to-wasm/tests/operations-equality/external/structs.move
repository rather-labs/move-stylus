module test::equality_external_structs;

use test::equality_external_structs_def::{
    Bar, Foo, create_foo_bool, create_foo_u8, create_foo_u16, create_foo_u32,
    create_foo_u64, create_foo_u128, create_foo_u256, create_foo_vec_stack_type,
    create_foo_vec_heap_type, create_foo_address, create_foo_struct
};

public fun eq_struct_bool(a: bool, b: bool): bool {
    let foo = create_foo_bool(a);
    let bar = create_foo_bool(b);

    foo == bar
}

public fun eq_struct_u8(a: u8, b: u8): bool {
    let foo = create_foo_u8(a);
    let bar = create_foo_u8(b);

    foo == bar
}

public fun eq_struct_u16(a: u16, b: u16): bool {
    let foo = create_foo_u16(a);
    let bar = create_foo_u16(b);

    foo == bar
}

public fun eq_struct_u32(a: u32, b: u32): bool {
    let foo = create_foo_u32(a);
    let bar = create_foo_u32(b);

    foo == bar
}

public fun eq_struct_u64(a: u64, b: u64): bool {
    let foo = create_foo_u64(a);
    let bar = create_foo_u64(b);

    foo == bar
}

public fun eq_struct_u128(a: u128, b: u128): bool {
    let foo = create_foo_u128(a);
    let bar = create_foo_u128(b);

    foo == bar
}

public fun eq_struct_u256(a: u256, b: u256): bool {
    let foo = create_foo_u256(a);
    let bar = create_foo_u256(b);

    foo == bar
}

public fun eq_struct_vec_stack_type(a: vector<u32>, b: vector<u32>): bool {
    let foo = create_foo_vec_stack_type(a);
    let bar = create_foo_vec_stack_type(b);

    foo == bar
}

public fun eq_struct_vec_heap_type(a: vector<u128>, b: vector<u128>): bool {
    let foo = create_foo_vec_heap_type(a);
    let bar = create_foo_vec_heap_type(b);

    foo == bar
}

public fun eq_struct_address(a: address, b: address): bool {
    let foo = create_foo_address(a);
    let bar = create_foo_address(b);

    foo == bar
}

public fun eq_struct_struct(a: u32, b: u128, c: u32, d: u128): bool {
    let foo = create_foo_struct(a, b);
    let bar = create_foo_struct(c, d);

    foo == bar
}

public fun neq_struct_bool(a: bool, b: bool): bool {
    let foo = create_foo_bool(a);
    let bar = create_foo_bool(b);

    foo != bar
}

public fun neq_struct_u8(a: u8, b: u8): bool {
    let foo = create_foo_u8(a);
    let bar = create_foo_u8(b);

    foo != bar
}

public fun neq_struct_u16(a: u16, b: u16): bool {
    let foo = create_foo_u16(a);
    let bar = create_foo_u16(b);

    foo != bar
}

public fun neq_struct_u32(a: u32, b: u32): bool {
    let foo = create_foo_u32(a);
    let bar = create_foo_u32(b);

    foo != bar
}

public fun neq_struct_u64(a: u64, b: u64): bool {
    let foo = create_foo_u64(a);
    let bar = create_foo_u64(b);

    foo != bar
}

public fun neq_struct_u128(a: u128, b: u128): bool {
    let foo = create_foo_u128(a);
    let bar = create_foo_u128(b);

    foo != bar
}

public fun neq_struct_u256(a: u256, b: u256): bool {
    let foo = create_foo_u256(a);
    let bar = create_foo_u256(b);

    foo != bar
}

public fun neq_struct_vec_stack_type(a: vector<u32>, b: vector<u32>): bool {
    let foo = create_foo_vec_stack_type(a);
    let bar = create_foo_vec_stack_type(b);

    foo != bar
}

public fun neq_struct_vec_heap_type(a: vector<u128>, b: vector<u128>): bool {
    let foo = create_foo_vec_heap_type(a);
    let bar = create_foo_vec_heap_type(b);

    foo != bar
}

public fun neq_struct_address(a: address, b: address): bool {
    let foo = create_foo_address(a);
    let bar = create_foo_address(b);

    foo != bar
}

public fun neq_struct_struct(a: u32, b: u128, c: u32, d: u128): bool {
    let foo = create_foo_struct(a, b);
    let bar = create_foo_struct(c, d);

    foo != bar
}
