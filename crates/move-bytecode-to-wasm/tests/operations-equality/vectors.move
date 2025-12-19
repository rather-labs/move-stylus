module 0x01::equality_vectors;

entry fun eq_vec_u8(x: vector<u8>, y: vector<u8>): bool {
    x == y
}

entry fun eq_vec_stack_type(x: vector<u16>, y: vector<u16>): bool {
    x == y
}

entry fun eq_vec_heap_type(x: vector<u128>, y: vector<u128>): bool {
    x == y
}

entry fun eq_vec_heap_type_2(x: vector<address>, y: vector<address>): bool {
    x == y
}

entry fun eq_vec_nested_stack_type(x: vector<vector<u16>>, y: vector<vector<u16>>): bool {
    x == y
}

entry fun eq_vec_nested_heap_type(x: vector<vector<u128>>, y: vector<vector<u128>>): bool {
    x == y
}

entry fun eq_vec_nested_heap_type_2(x: vector<vector<address>>, y: vector<vector<address>>): bool {
    x == y
}

entry fun neq_vec_stack_type(x: vector<u16>, y: vector<u16>): bool {
    x != y
}

entry fun neq_vec_heap_type(x: vector<u128>, y: vector<u128>): bool {
    x != y
}

entry fun neq_vec_heap_type_2(x: vector<address>, y: vector<address>): bool {
    x != y
}

entry fun neq_vec_nested_stack_type(x: vector<vector<u16>>, y: vector<vector<u16>>): bool {
    x != y
}

entry fun neq_vec_nested_heap_type(x: vector<vector<u128>>, y: vector<vector<u128>>): bool {
    x != y
}

entry fun neq_vec_nested_heap_type_2(x: vector<vector<address>>, y: vector<vector<address>>): bool {
    x != y
}

public struct Bar has drop {
    n: u32,
    o: u128,
}

entry fun eq_vec_bar(n1: u32, n2: u32, n3: u32, n4: u32): bool {
    let v1 = vector[Bar { n: n1, o: 1u128 }, Bar { n: n2, o: 2u128 }];
    let v2 = vector[Bar { n: n3, o: 1u128 }, Bar { n: n4, o: 2u128 }];
    v1 == v2
}
