module 0x01::bool_type;

const BOOL_AS_CONST: bool = true;

public fun not_true(): bool {
  !BOOL_AS_CONST
}

public fun not(x: bool): bool {
  !x
}
