//!
//! An instrumenting middleware for global allocators in Rust, useful in testing
//! for validating assumptions regarding allocation patterns, and potentially in
//! production loads to monitor for memory leaks.
//!
//! ## Example
//!
//! ```
//! extern crate stats_alloc;
//!
//! use stats_alloc::{StatsAlloc, Region};
//! use std::alloc::System;
//!
//! #[global_allocator]
//! static STATS_ALLOC: StatsAlloc<System> = StatsAlloc::system();
//!
//! fn main() {
//!     let reg = Region::new(&STATS_ALLOC);
//!     let x: Vec<u8> = Vec::with_capacity(1_024);
//!     println!("Stats at 1: {:#?}", reg.change());
//!     // Used here to esnure that the value isn't deallocated first
//!     ::std::mem::size_of_val(&x);
//! }
//! ```

#![feature(const_fn)]

use std::alloc::{GlobalAlloc, Layout, System};
use std::ops;
use std::sync::atomic::{AtomicUsize, AtomicIsize, Ordering};

#[derive(Default, Debug)]
pub struct StatsAlloc<T: GlobalAlloc> {
    allocations: AtomicUsize,
    deallocations: AtomicUsize,
    reallocations: AtomicUsize,
    bytes_allocated: AtomicUsize,
    bytes_deallocated: AtomicUsize,
    bytes_reallocated: AtomicIsize,
    inner: T,
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
pub struct Stats {
    allocations: usize,
    deallocations: usize,
    reallocations: usize,
    bytes_allocated: usize,
    bytes_deallocated: usize,
    bytes_reallocated: isize,
}

impl StatsAlloc<System> {
    pub const fn system() -> Self {
        Self::new(System)
    }
}

impl<T: GlobalAlloc> StatsAlloc<T> {
    pub const fn new(inner: T) -> Self {
        StatsAlloc {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            reallocations: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            bytes_deallocated: AtomicUsize::new(0),
            bytes_reallocated: AtomicIsize::new(0),
            inner,
        }
    }

    pub fn stats(&self) -> Stats {
        Stats {
            allocations: self.allocations.load(Ordering::SeqCst),
            deallocations: self.deallocations.load(Ordering::SeqCst),
            reallocations: self.reallocations.load(Ordering::SeqCst),
            bytes_allocated: self.bytes_allocated.load(Ordering::SeqCst),
            bytes_deallocated: self.bytes_deallocated.load(Ordering::SeqCst),
            bytes_reallocated: self.bytes_reallocated.load(Ordering::SeqCst),
        }
    }
}

impl ops::Sub for Stats {
    type Output = Stats;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl ops::SubAssign for Stats {
    fn sub_assign(&mut self, rhs: Self) {
        self.allocations -= rhs.allocations;
        self.deallocations -= rhs.deallocations;
        self.reallocations -= rhs.reallocations;
        self.bytes_allocated -= rhs.bytes_allocated;
        self.bytes_deallocated -= rhs.bytes_deallocated;
        self.bytes_reallocated -= rhs.bytes_reallocated;
    }
}

pub struct Region<'a, T: GlobalAlloc + 'a> {
    alloc: &'a StatsAlloc<T>,
    initial_stats: Stats,
}

impl<'a, T: GlobalAlloc + 'a> Region<'a, T> {
    pub fn new(alloc: &'a StatsAlloc<T>) -> Self {
        Region {
            alloc,
            initial_stats: alloc.stats(),
        }
    }

    pub fn initial(&self) -> Stats {
        self.initial_stats
    }

    pub fn change(&self) -> Stats {
        self.alloc.stats() - self.initial_stats
    }

    pub fn change_and_reset(&mut self) -> Stats {
        let latest = self.alloc.stats();
        let diff = latest - self.initial_stats;
        self.initial_stats = latest;
        diff
    }
}

unsafe impl<T: GlobalAlloc> GlobalAlloc for StatsAlloc<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocations.fetch_add(1, Ordering::SeqCst);
        self.bytes_allocated.fetch_add(layout.size(), Ordering::SeqCst);
        self.inner.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocations.fetch_add(1, Ordering::SeqCst);
        self.bytes_deallocated.fetch_add(layout.size(), Ordering::SeqCst);
        self.inner.dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.allocations.fetch_add(1, Ordering::SeqCst);
        self.bytes_allocated.fetch_add(layout.size(), Ordering::SeqCst);
        self.inner.alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        self.reallocations.fetch_add(1, Ordering::SeqCst);
        if new_size > layout.size() {
            self.bytes_allocated.fetch_add(new_size - layout.size(), Ordering::SeqCst);
        } else if new_size < layout.size() {
            self.bytes_deallocated.fetch_add(layout.size() - new_size, Ordering::SeqCst);
        }
        self.bytes_reallocated.fetch_add(new_size.wrapping_sub(layout.size()) as isize, Ordering::SeqCst);
        self.inner.realloc(ptr, layout, new_size)
    }
}
