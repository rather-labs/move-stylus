module 0x00::enum_abi_packing_unpacking;

public enum SimpleEnum has drop {
    One,
    Two,
    Three,
}

entry fun pack_1(): SimpleEnum {
    SimpleEnum::One
}

entry fun pack_2(): SimpleEnum {
    SimpleEnum::Two
}

entry fun pack_3(): SimpleEnum {
    SimpleEnum::Three
}

entry fun pack_unpack(x:  SimpleEnum): SimpleEnum {
    x
}
