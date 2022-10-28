extern crate stats_alloc;

use std::alloc::System;

use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};

#[global_allocator]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[test]
fn example_using_region() {
    let reg = Region::new(GLOBAL);
    let x: Vec<u8> = Vec::with_capacity(1_024);
    println!("Stats at 1: {:#?}", reg.change());
    // Used here to ensure that the value is not
    // dropped before we check the statistics
    let _ = ::std::mem::size_of_val(&x);
}

#[test]
fn example_peak_mem_usage() {
    let x: Vec<u8> = Vec::with_capacity(1_024);
    println!("Stats at 1: {:#?}", GLOBAL.peak_mem_usage());
    // Used here to ensure that the value is not
    // dropped before we check the statistics
    let _ = ::std::mem::size_of_val(&x);
}
