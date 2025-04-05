module 0x01::uint_16;

const ITEM_PRICE: u16 = 1616;

public fun get_const(): u16 {
  ITEM_PRICE
}

public fun echo(x: u16): u16 {
  identity(x)
}

public fun echo_2(x: u16, y: u16): u16 {
  identity_2(x, y)
}

fun identity(x: u16): u16 {
  x
}

fun identity_2(_x: u16, y: u16): u16 {
  y
}
