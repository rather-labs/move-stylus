module 0x00::enum_abi_packing_unpacking;

public enum SimpleEnum has drop {
    One,
    Two,
    Three,
}

public fun pack_1(): SimpleEnum {
    SimpleEnum::One
}

public fun pack_2(): SimpleEnum {
    SimpleEnum::Two
}

public fun pack_3(): SimpleEnum {
    SimpleEnum::Three
}

public fun pack_unpack(x:  SimpleEnum): SimpleEnum {
    x
}
