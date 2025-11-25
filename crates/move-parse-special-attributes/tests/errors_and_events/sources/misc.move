module test::misc;

#[ext(abi_error)]
public struct ErrorWithKey(String) has copy, drop, key;

#[ext(event)]
public struct EventWithKey(String) has key;

#[ext(abi_error)]
public struct ErrorOk(String) has drop;

#[ext(abi_error)]
public struct Error(String) has drop;

#[ext(event, indexes = 1)]
public struct EventOk has copy, drop {
    a: u64,
    b: u128,
}

entry fun revert_error(a: String) {
    revert(ErrorOk(a));
}

entry fun revert_error_bad_arg(error: ErrorOk) {
    revert(error);
}

entry fun emit_event(a: u64, b: u128) {
    emit(EventOk { a, b });
}

entry fun emit_event_bad_arg(event: EventOk) {
    emit(event);
}

entry fun revert_error_invalid_name(s: String) {
    revert(Error(s));
}