module 0x00::struct_misc;

public struct Empty has drop {}

#[allow(unused_field)]
public struct Tuple(u32, vector<u8>) has drop;

#[allow(unused_field)]
public struct TupleGeneric<T>(T, vector<u8>) has drop;

entry fun pack_unpack_abi_empty(e: Empty): Empty {
    e
}

entry fun pack_unpack_abi_tuple(t: Tuple): Tuple {
    t
}

entry fun pack_unpack_abi_tuple_generic(t: TupleGeneric<u64>): TupleGeneric<u64> {
    t
}

// Usage of phantoms and empty structs
public struct USD has drop {}

public struct JPY has drop {}

public struct Coin<phantom T> has drop {
    amount: u64,
}

entry fun exchange_usd_to_jpy(usd: Coin<USD>): Coin<JPY> {
    Coin<JPY> { amount: usd.amount * 150 }
}
