module hello_world::hello_world;

use hello_world::stack::Stack;
use hello_world::stack;

public fun test_stack_1(): Stack<u32> {
    let mut s = stack::new(vector[]);
    s.push_back(1);
    s
}
