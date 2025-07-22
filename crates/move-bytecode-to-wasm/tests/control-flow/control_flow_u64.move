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


public fun fibonacci(n: u64): u64 {
    if (n == 0) return 0;
    if (n == 1) return 1;
    let mut a = 0;
    let mut b = 1;
    let mut count = 2;
    while (count <= n) {
        let temp = a + b;
        a = b;
        b = temp;
        count = count + 1;
    };
    b
}


public fun is_prime(i: u64): u64 {
    if (i > 1) {
        let mut j = 2;
        let mut is_prime = 1;
        while (j * j <= i) {
                if (i % j == 0) {
                    is_prime = 0;
                    break
                };
                j = j + 1;
        };
        is_prime
    } else {
        0
    }
}

public fun sum_special(n: u64): u64 {
    let mut total = 0;
    let mut i = 1;

    'outer: loop {
        if (i > n) {
            break // Exit main loop
        };

        // Check if i is prime using a while loop
        if (i > 1) {
            let mut j = 2;
            let mut x = 1;
            while (j * j <= i) {
                if (i % j == 0) {
                    x = 0;
                    break
                };
                j = j + 1;
            };

            if (x == 1) {
                total = total + 7;
            };
        };

        i = i + 1;
    };

    total
}
