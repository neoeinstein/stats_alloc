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
//! use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
//! use std::alloc::System;
//!
//! #[global_allocator]
//! static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;
//!
//! fn main() {
//!     let reg = Region::new(&GLOBAL);
//!     let x: Vec<u8> = Vec::with_capacity(1_024);
//!     println!("Stats at 1: {:#?}", reg.change());
//!     // Used here to ensure that the value is not
//!     // dropped before we check the statistics
//!     ::std::mem::size_of_val(&x);
//! }
//! ```

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_imports,
    unused_qualifications,
    missing_docs
)]
#![cfg_attr(feature = "docs-rs", feature(allocator_api))]

use std::{
    alloc::{GlobalAlloc, Layout, System},
    ops,
    sync::atomic::{AtomicIsize, AtomicUsize, Ordering},
};

/// An instrumenting middleware which keeps track of allocation, deallocation,
/// and reallocation requests to the underlying global allocator.
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

/// Allocator statistics
#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
pub struct Stats {
    /// Count of allocation operations
    pub allocations: usize,
    /// Count of deallocation operations
    pub deallocations: usize,
    /// Count of reallocation operations
    ///
    /// An example where reallocation may occur: resizing of a `Vec<T>` when
    /// its length would excceed its capacity. Excessive reallocations may
    /// indicate that resizable data structures are being created with
    /// insufficient or poorly estimated initial capcities.
    ///
    /// ```
    /// let mut x = Vec::with_capacity(1);
    /// x.push(0);
    /// x.push(1); // Potential reallocation
    /// ```
    pub reallocations: usize,
    /// Total bytes requested by allocations
    pub bytes_allocated: usize,
    /// Total bytes freed by deallocations
    pub bytes_deallocated: usize,
    /// Total of bytes requested minus bytes freed by reallocations
    ///
    /// This number is positive if the total bytes requested by reallocation
    /// operations is greater than the total bytes freed by reallocations. A
    /// positive value indicates that resizable structures are growing, while
    /// a negative value indicates that such structures are shrinking.
    pub bytes_reallocated: isize,
}

/// An instrumented instance of the system allocator.
pub static INSTRUMENTED_SYSTEM: StatsAlloc<System> = StatsAlloc {
    allocations: AtomicUsize::new(0),
    deallocations: AtomicUsize::new(0),
    reallocations: AtomicUsize::new(0),
    bytes_allocated: AtomicUsize::new(0),
    bytes_deallocated: AtomicUsize::new(0),
    bytes_reallocated: AtomicIsize::new(0),
    inner: System,
};

impl StatsAlloc<System> {
    /// Provides access to an instrumented instance of the system allocator.
    pub const fn system() -> Self {
        StatsAlloc {
            allocations: AtomicUsize::new(0),
            deallocations: AtomicUsize::new(0),
            reallocations: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            bytes_deallocated: AtomicUsize::new(0),
            bytes_reallocated: AtomicIsize::new(0),
            inner: System,
        }
    }
}

impl<T: GlobalAlloc> StatsAlloc<T> {
    /// Provides access to an instrumented instance of the given global
    /// allocator.
    #[cfg(feature = "nightly")]
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

    /// Provides access to an instrumented instance of the given global
    /// allocator.
    #[cfg(not(feature = "nightly"))]
    pub fn new(inner: T) -> Self {
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

    /// Takes a snapshot of the current view of the allocator statistics.
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

/// A snapshot of the allocation statistics, which can be used to determine
/// allocation changes while the `Region` is alive.
#[derive(Debug)]
pub struct Region<'a, T: GlobalAlloc + 'a> {
    alloc: &'a StatsAlloc<T>,
    initial_stats: Stats,
}

impl<'a, T: GlobalAlloc + 'a> Region<'a, T> {
    /// Creates a new region using statistics from the given instrumented
    /// allocator.
    #[inline]
    pub fn new(alloc: &'a StatsAlloc<T>) -> Self {
        Region {
            alloc,
            initial_stats: alloc.stats(),
        }
    }

    /// Returns the statistics as of instantiation or the last reset.
    #[inline]
    pub fn initial(&self) -> Stats {
        self.initial_stats
    }

    /// Returns the difference between the currently reported statistics and
    /// those provided by `initial()`.
    #[inline]
    pub fn change(&self) -> Stats {
        self.alloc.stats() - self.initial_stats
    }

    /// Returns the difference between the currently reported statistics and
    /// those provided by `initial()`, resetting initial to the latest
    /// reported statistics.
    #[inline]
    pub fn change_and_reset(&mut self) -> Stats {
        let latest = self.alloc.stats();
        let diff = latest - self.initial_stats;
        self.initial_stats = latest;
        diff
    }

    /// Resets the initial initial to the latest reported statistics from the
    /// referenced allocator.
    #[inline]
    pub fn reset(&mut self) {
        self.initial_stats = self.alloc.stats();
    }
}

unsafe impl<'a, T: GlobalAlloc + 'a> GlobalAlloc for &'a StatsAlloc<T> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (*self).alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (*self).dealloc(ptr, layout)
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        (*self).alloc_zeroed(layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        (*self).realloc(ptr, layout, new_size)
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
            let difference = new_size - layout.size();
            self.bytes_allocated.fetch_add(difference, Ordering::SeqCst);
        } else if new_size < layout.size() {
            let difference = layout.size() - new_size;
            self.bytes_deallocated.fetch_add(difference, Ordering::SeqCst);
        }
        self.bytes_reallocated
            .fetch_add(new_size.wrapping_sub(layout.size()) as isize, Ordering::SeqCst);
        self.inner.realloc(ptr, layout, new_size)
    }
}
