module 0x01::equality_vectors;

/*
public fun eq_vec_stack_type(x: vector<u16>, y: vector<u16>): bool {
    x == y
}

public fun eq_vec_heap_type(x: vector<u128>, y: vector<u128>): bool {
    x == y
}

public fun eq_vec_nested_stack_type(x: vector<vector<u16>>, y: vector<vector<u16>>): bool {
    x == y
}
*/
public fun eq_vec_nested_heap_type(x: vector<vector<u128>>, y: vector<vector<u128>>): bool {
    x == y
}
