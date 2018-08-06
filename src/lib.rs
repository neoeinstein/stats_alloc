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
//! use stats_alloc::{Region, Instrumented, THREAD_INSTRUMENTED_SYSTEM};
//! use std::alloc::System;
//!
//! #[global_allocator]
//! static GLOBAL: &Instrumented<System> = &THREAD_INSTRUMENTED_SYSTEM;
//!
//! fn main() {
//!     let reg = Region::new();
//!     let x: Vec<u8> = Vec::with_capacity(1_024);
//!     println!("Stats at 1: {:#?}", reg.change());
//!     // Used here to ensure that the value is not
//!     // dropped before we check the statistics
//!     ::std::mem::size_of_val(&x);
//! }
//! ```

#![deny(
    bare_trait_object,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub
    unused_extern_crates,
    unused_import_braces,
    unused_imports,
    unused_qualifications,
)]
#![cfg_attr(feature = "nightly", feature(const_fn))]
#![cfg_attr(feature = "docs-rs", feature(allocator_api))]

extern crate spin;
#[cfg(not(feature = "nightly"))]
#[macro_use]
extern crate lazy_static;

mod accum;
mod global_alloc;
mod region;
mod summary;

pub use accum::{
    rollup::Rollup,
    reporter::{AlwaysReport, IntervalReport, LocalAlwaysReport, LocalIntervalReport, NeverReport, Reporter},
    thread_local::ThreadStats,
};
pub use global_alloc::instrumented::{
    FULLY_INSTRUMENTED_SYSTEM,
    INSTRUMENTED_SYSTEM_WITH_1_SEC_ROLLUP,
    THREAD_INSTRUMENTED_SYSTEM,
    Instrumented,
};
pub use region::Region;
pub use summary::Summary;
