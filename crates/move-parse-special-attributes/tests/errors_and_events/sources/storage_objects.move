module test::storage_objects;

public struct TestStruct(u32) has key;

public struct TestStruct2(u32) has drop;

#[ext(owned_objects(s))]
entry fun read_test_struct_1(s: &TestStruct): u32 {
    s.0
}

// Missing owned object
#[ext(owned_objects(t))]
entry fun read_test_struct_2(s: &TestStruct): u32 {
    s.0
}

// Missing owned object with other correct ones
#[ext(owned_objects(s, t))]
entry fun read_test_struct_4(s: &TestStruct): u32 {
    s.0
}

// Shared objects tests
#[ext(shared_objects(s))]
entry fun read_test_struct_shared_1(s: &TestStruct): u32 {
    s.0
}

// Missing shared object
#[ext(shared_objects(t))]
entry fun read_test_struct_shared_2(s: &TestStruct): u32 {
    s.0
}

// Missing shared object with other correct ones
#[ext(shared_objects(s, t))]
entry fun read_test_struct_shared_4(s: &TestStruct): u32 {
    s.0
}

// Frozen objects tests
#[ext(frozen_objects(s))]
entry fun read_test_struct_frozen_1(s: &TestStruct): u32 {
    s.0
}

// Missing frozen object
#[ext(frozen_objects(t))]
entry fun read_test_struct_frozen_2(s: &TestStruct): u32 {
    s.0
}

// Missing frozen object with other correct ones
#[ext(frozen_objects(s, t))]
entry fun read_test_struct_frozen_4(s: &TestStruct): u32 {
    s.0
}

// Missing and repeated frozen object
#[ext(frozen_objects(s, t))]
entry fun read_test_struct_frozen_5(s: &TestStruct): u32 {
    s.0
}

// Mixed storage objects tests
// Owned + Shared
#[ext(owned_objects(s), shared_objects(t))]
entry fun read_test_struct_mixed_1(s: &TestStruct, t: &TestStruct): (u32, u32) {
    (
        s.0,
        t.0,
    )
}

// Owned + Frozen
#[ext(owned_objects(s), frozen_objects(t))]
entry fun read_test_struct_mixed_2(s: &TestStruct, t: &TestStruct): (u32, u32) {
    (
        s.0,
        t.0,
    )
}

// Shared + Frozen
#[ext(shared_objects(s), frozen_objects(t))]
entry fun read_test_struct_mixed_3(s: &TestStruct, t: &TestStruct): (u32, u32) {
    (
        s.0,
        t.0,
    )
}

// All three types
#[ext(owned_objects(s), shared_objects(t), frozen_objects(u))]
entry fun read_test_struct_mixed_4(s: &TestStruct, t: &TestStruct, u: &TestStruct): (u32, u32, u32) {
    (
        s.0,
        t.0,
        u.0
    )
}

// Repeated across different types (s appears in both owned and shared)
#[ext(owned_objects(s), shared_objects(s))]
entry fun read_test_struct_mixed_5(s: &TestStruct): u32 {
    s.0
}

// Missing parameter in mixed declaration
#[ext(owned_objects(s), shared_objects(t))]
entry fun read_test_struct_mixed_6(s: &TestStruct): u32 {
    s.0
}

// Repeated within one type and also across types
#[ext(owned_objects(s), shared_objects(s))]
entry fun read_test_struct_mixed_7(s: &TestStruct): u32 {
    s.0
}

// Multiple missing and repeated across types
#[ext(owned_objects(s, t), shared_objects(u, v), frozen_objects(s))]
entry fun read_test_struct_mixed_8(s: &TestStruct): u32 {
    s.0
}

#[ext(owned_objects(s))]
entry fun read_test_struct_no_keyed_1(s: &TestStruct2): u32 {
    s.0
}

#[ext(shared_objects(s))]
entry fun read_test_struct_no_keyed_2(s: &TestStruct2): u32 {
    s.0
}

#[ext(frozen_objects(s))]
entry fun read_test_struct_no_keyed_3(s: &TestStruct2): u32 {
    s.0
}
