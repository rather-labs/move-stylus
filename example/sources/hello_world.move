module 0x01::hello_world;

public fun vec_len(x: vector<u32>): u64 {
  x.length()
}

public fun vec_len_2(x: vector<vector<u128>>): u64 {
  x.length()
}