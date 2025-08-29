module hello_world::hello_world;

use hello_world::stack::{Stack, new};

public fun stack_usage() {
    let s = new(vector[1,2,3]);
}
