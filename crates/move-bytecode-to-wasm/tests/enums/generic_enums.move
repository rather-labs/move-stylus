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

public enum NesterGenericEnum<T> has drop {
    Variant1(T),
    Variant2(T),
    Variant3{x: T},
}

public enum WrappedGenericEnum<T> has drop {
    Variant1(NesterGenericEnum<T>),
    Variant2(vector<NesterGenericEnum<T>>),
    Variant3{x: NesterGenericEnum<T>, y: NesterGenericEnum<T>},
}

fun create_wrapped_generic_enum<T: drop + copy>(variant_index: u8, x: T): WrappedGenericEnum<T> {
    match (variant_index) {
        0 => WrappedGenericEnum<T>::Variant1(NesterGenericEnum<T>::Variant1(x)),
        1 => WrappedGenericEnum<T>::Variant2(vector[NesterGenericEnum<T>::Variant1(x), NesterGenericEnum<T>::Variant2(x), NesterGenericEnum<T>::Variant3{x}]),
        2 => WrappedGenericEnum<T>::Variant3{x: NesterGenericEnum<T>::Variant1(x), y: NesterGenericEnum<T>::Variant2(x)},
        _ => abort(1),
    }
}

fun unpack_nested_generic_enum_u32(enum_: NesterGenericEnum<u32>): u32 {
    match (enum_) {
        NesterGenericEnum<u32>::Variant1(value) => value,
        NesterGenericEnum<u32>::Variant2(value) => value,
        NesterGenericEnum<u32>::Variant3{x} => x,
    }
}

entry fun create_wrapped_generic_enum_u32(variant_index: u8, x: u32): u32 {
    let enum_ = create_wrapped_generic_enum(variant_index, x);
    match (enum_) {
        WrappedGenericEnum<u32>::Variant1(nested) => {
            unpack_nested_generic_enum_u32(nested)
        },
        WrappedGenericEnum<u32>::Variant2(nested_vector) => {
            let mut sum = 0;
            let mut vec_copy = nested_vector;
            while (std::vector::length(&vec_copy) > 0) {
                let nested = std::vector::pop_back(&mut vec_copy);
                sum = sum + unpack_nested_generic_enum_u32(nested);
            };
            sum
        },
        WrappedGenericEnum<u32>::Variant3{x, y} => {
            unpack_nested_generic_enum_u32(x) + unpack_nested_generic_enum_u32(y)
        },
    }
}