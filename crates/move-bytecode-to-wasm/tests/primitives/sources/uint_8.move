module 0x01::uint_8;

const ITEM_PRICE: u8 = 88;

public fun get_const(): u8 {
  ITEM_PRICE
}

public fun echo(x: u8): u8 {
  identity(x)
}

public fun echo_2(x: u8, y: u8): u8 {
  identity_2(x, y)
}

fun identity(x: u8): u8 {
  x
}

fun identity_2(_x: u8, y: u8): u8 {
  y
}
