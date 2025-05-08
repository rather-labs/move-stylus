module 0x01::multi_values_return;

const INT_U256: u256 = 256256;
const INT_U64: u64 = 6464;
const INT_U32: u32 = 3232;
const INT_U8: u8 = 88;
const INT_BOOL: bool = true;
const ADDRESS_AS_CONST: address = @0x01;
const VECTOR_U32: vector<u32> = vector[10, 20, 30];
const VECTOR_U128: vector<u128> = vector[100, 200, 300];

public fun get_constants(): (u256, u64, u32, u8, bool, address, vector<u32>, vector<u128>) {
  (INT_U256, INT_U64, INT_U32, INT_U8, INT_BOOL, ADDRESS_AS_CONST, VECTOR_U32, VECTOR_U128)
}

public fun get_constants_reversed(): (vector<u128>, vector<u32>, address, bool, u8, u32, u64, u256) {
  (VECTOR_U128, VECTOR_U32, ADDRESS_AS_CONST, INT_BOOL, INT_U8, INT_U32, INT_U64, INT_U256)
}

public fun get_constants_nested(): (u256, u64, u32, u8, bool, address, vector<u32>, vector<u128>) {
  get_constants_inner()
}

fun get_constants_inner(): (u256, u64, u32, u8, bool, address, vector<u32>, vector<u128>) {
  (INT_U256, INT_U64, INT_U32, INT_U8, INT_BOOL, ADDRESS_AS_CONST, VECTOR_U32, VECTOR_U128)
}
