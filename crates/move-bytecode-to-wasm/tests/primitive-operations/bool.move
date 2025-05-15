module 0x01::bool_type;

const BOOL_AS_CONST: bool = true;

//public fun and_true(x: bool): bool {
//  x && BOOL_AS_CONST
//}
//
//public fun and(x: bool, y: bool): bool {
//  x && y
//}
//
//public fun or(x: bool, y: bool): bool {
//  x || y
//}

public fun not_true(): bool {
  !BOOL_AS_CONST
}

public fun not(x: bool): bool {
  !x
}
