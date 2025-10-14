module 0x00::enum_with_fields;

public enum Bar has drop {
    VariantWithPositionalFields1(u64, bool),
    VariantWithPositionalFields2(u8, u16, u32),
}

public fun pack_bar_1(u: u64, b: bool): Bar {
    Bar::VariantWithPositionalFields1(u, b)
}

public fun pack_bar_2(u: u8, v: u16, w: u32): Bar {
    Bar::VariantWithPositionalFields2(u, v, w)
}

// public enum Baz has drop {
//     VariantWithNamedFields1 { x: u64, y: bool, z: Bar },
//     VariantWithNamedFields2 { x: u8, y: u16, z: u32 },
// }

// public fun pack_baz_1(x: u64, y: bool, z: Bar): Baz {
//     Baz::VariantWithNamedFields1 { x, y, z }
// }

// public fun pack_baz_2(x: u8, y: u16, z: u32): Baz {
//     Baz::VariantWithNamedFields2 { x, y, z }
// }