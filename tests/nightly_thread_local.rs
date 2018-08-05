#![cfg(feature = "nightly")]

extern crate stats_alloc;

use stats_alloc::{Instrumented, LocalAlwaysReport, ThreadStats, Rollup};
use std::alloc::System;

#[global_allocator]
static GLOBAL: Instrumented<System> = Instrumented::system();

#[test]
fn test1() -> ::std::thread::Result<()> {
    ::std::thread::spawn(|| {
        let s1 = ThreadStats::summary();
        let x: Vec<u8> = Vec::with_capacity(1_024);
        let s2 = ThreadStats::summary();
        println!("Test1:\nStats at 1: {:#?}\nStats at 2: {:#?}\nDiff: {:#?}", s1, s2, s2 - s1);
        ::std::mem::size_of_val(&x);
    }).join()
}

#[test]
fn test2() -> ::std::thread::Result<()> {
    ::std::thread::spawn(|| {
        let s1 = ThreadStats::summary();
        for i in 0..10_000 {
            let x: Vec<u8> = Vec::with_capacity(i);
        }
        let s2 = ThreadStats::summary();
        println!("Test2:\nStats at 1: {:#?}\nStats at 2: {:#?}\nDiff: {:#?}", s1, s2, s2 - s1);
    }).join()
}

#[test]
fn drop_test() {
    let st1 = ThreadStats::summary();
    let sg1 = Rollup::global().summary();
    let t = ::std::thread::spawn(|| {
        let st1 = ThreadStats::summary();
        let x: Vec<u8> = Vec::with_capacity(1_024);
        let st2 = ThreadStats::summary();
        let tid = ::std::thread::current().id();
        (st1, st2, tid)
    });
    let x = Vec::<u8>::with_capacity(2_049);
    let (si1, si2, tid) = t.join().unwrap();
    ThreadStats::flush();
    let st2 = ThreadStats::summary();
    let sg2 = Rollup::global().summary();
    println!("Drop Test Thread:\nStats at 1: {:#?}\nStats at 2: {:#?}\nDiff: {:#?}", st1, st2, st2 - st1);
    println!("Drop Test Inner:\nStats at 1: {:#?}\nStats at 2: {:#?}\nDiff: {:#?}", si1, si2, si2 - si1);
    println!("Drop Test Global:\nStats at 1: {:#?}\nStats at 2: {:#?}\nDiff: {:#?}", sg1, sg2, sg2 - sg1);
    println!("Inner: {:?}, Outer: {:?}", tid, ::std::thread::current().id());
    debug_assert_eq!(1_024, (si2 - si1).total_bytes_allocated());
    debug_assert_eq!(1_024, (si2 - si1).outstanding_bytes_allocated());
}

#[test]
fn thread_size_test() {
    static ROLLUP: Rollup = Rollup::new();
    static LOCAL_REPORTER: LocalAlwaysReport = LocalAlwaysReport::new(&ROLLUP);
    ThreadStats::set_thread_reporter(&LOCAL_REPORTER);
    let sg1 = ROLLUP.summary();
    let st1 = ThreadStats::summary();
//    for _ in 0..128 {
        ::std::thread::spawn(|| {
            ThreadStats::set_thread_reporter(&LOCAL_REPORTER);
        }).join().unwrap();
//    }
    let st2 = ThreadStats::summary();
    let sg2 = ROLLUP.summary();
    ThreadStats::flush();
    let stf = ROLLUP.summary();
    println!("Thread Size Test Thread:\nStats at 1: {:#?}\nStats at 2: {:#?}\nDiff: {:#?}", st1, st2, st2 - st1);
    println!("Thread Size Test Global:\nStats at 1: {:#?}\nStats at 2: {:#?}\nDiff: {:#?}", sg1, sg2, sg2 - sg1);
    println!("STF: {:#?}", stf);
    assert_eq!(0, stf.outstanding_bytes_allocated());
}
