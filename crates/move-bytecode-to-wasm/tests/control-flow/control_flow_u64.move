module 0x01::control_flow_u64;

public fun collatz(mut x: u64): u64 {
    let mut count = 0;
    while (x != 1) {
        if (x % 2 == 0) {
            x = x / 2;
        } else {
            x = x * 3 + 1;
        };
        count = count + 1;
    };
    count
}