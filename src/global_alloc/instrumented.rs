use std::{
    alloc::{GlobalAlloc, Layout, System},
    time::Duration,
};
use accum::{
    reporter::{AlwaysReport, IntervalReport, NeverReport, Reporter},
    thread_local::ThreadStats,
};

/// An instrumented instance of the system allocator which always rolls up allocation statistics to the global accumulator with every call to the allocation API.
pub static FULLY_INSTRUMENTED_SYSTEM: Instrumented<System, AlwaysReport> = Instrumented {
    inner: System,
    default_reporter: AlwaysReport,
};

/// An instrumented instance of the system allocator which never rolls up allocation statistics to the global accumulator during the running lifetime of a thread.
///
/// Final allocation statistics for a thread are always reported when that thread is dropped.
pub static THREAD_INSTRUMENTED_SYSTEM: Instrumented<System, NeverReport> = Instrumented {
    inner: System,
    default_reporter: NeverReport,
};

/// An instrumented instance of the system allocator which rolls up allocation statistics to the global accumulator when an allocation occurs at least one second after a previous report.
///
/// Final allocation data for a thread is rolled up when the thread is dropped.
pub static INSTRUMENTED_SYSTEM_WITH_1_SEC_ROLLUP: Instrumented<System, IntervalReport> = Instrumented {
    inner: System,
    default_reporter: IntervalReport(Duration::from_secs(1)),
};

/// An instrumenting middleware which keeps track of allocation, deallocation,
/// and reallocation requests to the underlying global allocator.
#[derive(Default, Debug)]
pub struct Instrumented<T: GlobalAlloc, I: Reporter = NeverReport> {
    inner: T,
    default_reporter: I,
}

impl Instrumented<System> {
    /// Provides access to an instrumented instance of the system allocator.
    #[cfg(feature = "nightly")]
    pub const fn system() -> Self {
        Self::new(System, NeverReport)
    }

    /// Provides access to an instrumented instance of the system allocator.
    #[cfg(not(feature = "nightly"))]
    pub fn system() -> Self {
        Self::new(System, NeverReport)
    }
}

impl Instrumented<System, AlwaysReport> {
    /// Provides access to an instrumented instance of the system allocator.
    #[cfg(feature = "nightly")]
    pub const fn system_always_report() -> Self {
        Self::new(System, AlwaysReport)
    }

    /// Provides access to an instrumented instance of the system allocator.
    #[cfg(not(feature = "nightly"))]
    pub fn system_always_report() -> Self {
        Self::new(System, AlwaysReport)
    }
}

impl Instrumented<System, IntervalReport> {
    /// Provides access to an instrumented instance of the system allocator.
    #[cfg(feature = "nightly")]
    pub const fn system_report_every(interval: Duration) -> Self {
        Self::new(System, IntervalReport::new(interval))
    }

    /// Provides access to an instrumented instance of the system allocator.
    #[cfg(not(feature = "nightly"))]
    pub fn system_report_every(interval: Duration) -> Self {
        Self::new(System, IntervalReport::new(interval))
    }
}

impl<T: GlobalAlloc, I: Reporter> Instrumented<T, I> {
    /// TODO
    #[cfg(feature = "nightly")]
    pub const fn new(inner: T, default_reporter: I) -> Self {
        Self {
            inner,
            default_reporter,
        }
    }

    /// TODO
    #[cfg(not(feature = "nightly"))]
    pub fn new(inner: T, default_reporter: I) -> Self {
        Self {
            inner,
            default_reporter,
        }
    }
}

unsafe impl<'a, T: GlobalAlloc + 'a, I: Reporter> GlobalAlloc for &'a Instrumented<T, I> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        (*self).alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        (*self).dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        (*self).alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        (*self).realloc(ptr, layout, new_size)
    }
}

unsafe impl<T: GlobalAlloc, I: Reporter> GlobalAlloc for Instrumented<T, I> {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ThreadStats::alloc(layout, &self.default_reporter);
        self.inner.alloc(layout)
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ThreadStats::dealloc(layout, &self.default_reporter);
        self.inner.dealloc(ptr, layout)
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ThreadStats::alloc(layout, &self.default_reporter);
        self.inner.alloc_zeroed(layout)
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ThreadStats::realloc(layout, new_size, &self.default_reporter);
        self.inner.realloc(ptr, layout, new_size)
    }
}
