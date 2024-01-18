# stats_alloc

An instrumenting middleware for global allocators in Rust, useful in testing
for validating assumptions regarding allocation patterns, and potentially in
production loads to monitor for memory leaks.

## Example

```rust
extern crate stats_alloc;

use stats_alloc::{StatsAlloc, Region};
use std::alloc::System;

#[global_allocator]
static GLOBAL: StatsAlloc<System> = StatsAlloc::system();

fn example_using_region() {
    let reg = Region::new(&GLOBAL);
    let x: Vec<u8> = Vec::with_capacity(1_024);
    println!("Stats at 1: {:#?}", reg.change());
    // Used here to ensure that the value is not
    // dropped before we check the statistics
    let _ = std::mem::size_of_val(&x);
}
```
