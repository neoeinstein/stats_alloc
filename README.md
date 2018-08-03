# stats_alloc

An instrumenting middleware for global allocators in Rust, useful in testing
for validating assumptions regarding allocation patterns, and potentially in
production loads to monitor for memory leaks.

## Example

```rust
extern crate stats_alloc;

use stats_alloc::{StatsAlloc, Region, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn example_using_region() {
    let reg = Region::new(&GLOBAL);
    let x: Vec<u8> = Vec::with_capacity(1_024);
    println!("Stats at 1: {:#?}", reg.change());
    // Used here to ensure that the value is not
    // dropped before we check the statistics
    ::std::mem::size_of_val(&x);
}
``` 

## Custom allocators

Currenty wrapping a custom allocator requires the use of the nightly compiler
and compiling with the "nightly" feature due to the use of the unstable
`const_fn` and the fact that the internals of the instrumenting type are not
public. If that's fine with you, a custom allocator can be wrapped as follows:

```rust
#[global_allocator]
static GLOBAL: StatsAlloc<System> = StatsAlloc::new(MyCustomAllocator::new());
```
