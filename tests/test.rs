extern crate stats_alloc;

use stats_alloc::{Region, Instrumented, THREAD_INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[global_allocator]
static GLOBAL: &Instrumented<System> = &THREAD_INSTRUMENTED_SYSTEM;

#[test]
fn example_using_region() {
    let reg = Region::new();
    let x: Vec<u8> = Vec::with_capacity(1_024);
    println!("Stats at 1: {:#?}", reg.change());
    // Used here to ensure that the value is not
    // dropped before we check the statistics
    ::std::mem::size_of_val(&x);
}
