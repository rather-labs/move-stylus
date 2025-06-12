module 0x01::equality_references;

public fun eq_u256(x: u256, y: u256): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_u128(x: u128, y: u128): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_u64(x: u64, y: u64): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_u32(x: u32, y: u32): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_u16(x: u16, y: u16): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_u8(x: u8, y: u8): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_address(x: address, y: address): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_vec_stack_type(x: vector<u16>, y: vector<u16>): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_vec_heap_type(x: vector<u128>, y: vector<u128>): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_vec_nested_stack_type(x: vector<vector<u16>>, y: vector<vector<u16>>): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun eq_vec_nested_heap_type(x: vector<vector<u128>>, y: vector<vector<u128>>): bool {
    let w = &x;
    let z = &y;

    w == z
}

public fun neq_u256(x: u256, y: u256): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_u128(x: u128, y: u128): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_u64(x: u64, y: u64): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_u32(x: u32, y: u32): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_u16(x: u16, y: u16): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_u8(x: u8, y: u8): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_address(x: address, y: address): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_vec_stack_type(x: vector<u16>, y: vector<u16>): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_vec_heap_type(x: vector<u128>, y: vector<u128>): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_vec_nested_stack_type(x: vector<vector<u16>>, y: vector<vector<u16>>): bool {
    let w = &x;
    let z = &y;

    w != z
}

public fun neq_vec_nested_heap_type(x: vector<vector<u128>>, y: vector<vector<u128>>): bool {
    let w = &x;
    let z = &y;

    w != z
}
