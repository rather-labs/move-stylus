module 0x00::generic_enums;

public enum GenericEnum<T, U> has drop {
    Variant1(T),
    Variant2(U),
    Variant3{x: T, y: U},
}

fun create_generic_enum<T: drop, U: drop>(variant_index: u8, v0: T, v1: U): GenericEnum<T, U> {
    match (variant_index) {
        0 => GenericEnum<T, U>::Variant1(v0),
        1 => GenericEnum<T, U>::Variant2(v1),
        2 => GenericEnum<T, U>::Variant3{x: v0, y: v1},
        _ => abort(1),
    }
}

entry fun pack_unpack_generic_enum_u64_u32(variant_index: u8, v0: u64, v1: u32): (u64, u32) {
    let enum_ = create_generic_enum(variant_index, v0, v1);
    match (enum_) {
        GenericEnum::Variant1(value) => (value, v1),
        GenericEnum::Variant2(value) => (v0, value),
        GenericEnum<u64, u32>::Variant3{x, y} => (x, y),
    }
}

entry fun pack_unpack_generic_enum_u128_u16(variant_index: u8, v0: u128, v1: u16): (u128, u16) {
    let enum_ = create_generic_enum(variant_index, v0, v1);
    match (enum_) {
        GenericEnum::Variant1(value) => (value, v1),
        GenericEnum<u128, u16>::Variant2(value) => (v0, value),
        GenericEnum<u128, u16>::Variant3{x, y} => (x, y),
    }
}

// entry fun pack_unpack_generic_enum_u64_u32_(variant_index: u8, value64: u64, value32: u32): (u64, u32) {
//     let enum_ = match (variant_index) {
//         0 => GenericEnum<u64, u32>::Variant1(value64),
//         1 => GenericEnum<u64, u32>::Variant2(value32),
//         2 => GenericEnum<u64, u32>::Variant3{x: value64, y: value32},
//         _ => abort(1),
//     };

//     match (enum_) {
//         GenericEnum::Variant1(value) => (value, value32),
//         GenericEnum<u64, u32>::Variant2(value) => (value64, value),
//         GenericEnum<u64, u32>::Variant3{x, y} => (x, y),
//     }
// }
