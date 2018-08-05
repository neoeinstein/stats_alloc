use std::time::{Duration, Instant};
use accum::{
    rollup::Rollup,
    thread_local::ThreadStats,
};

/// TODO
pub trait Reporter {
    /// TODO
    fn report(&self, thread_stats: &mut ThreadStats);

    /// TODO
    #[inline(always)]
    fn rollup(&self) -> &Rollup {
        Rollup::global()
    }
}

/// TODO
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NeverReport;

impl Reporter for NeverReport {
    #[inline(always)]
    fn report(&self, _thread_stats: &mut ThreadStats) {}
}

/// TODO
#[derive(Clone, Copy, Debug)]
pub struct IntervalReport(pub(crate) Duration);

impl IntervalReport {
    /// TODO
    #[cfg(feature = "nightly")]
    pub const fn new(interval: Duration) -> Self {
        IntervalReport(interval)
    }

    /// TODO
    #[cfg(not(feature = "nightly"))]
    pub fn new(interval: Duration) -> Self {
        IntervalReport(interval)
    }
}

impl Reporter for IntervalReport {
    #[inline]
    fn report(&self, thread_stats: &mut ThreadStats) {
        if let Some(lu) = thread_stats.last_updated() {
            let now = Instant::now();
            if now - lu > self.0 {
                thread_stats.flush_to(self.rollup());
                thread_stats.set_last_updated(now);
            }
        } else {
            thread_stats.set_last_updated(Instant::now());
        }
    }

    #[inline]
    fn rollup(&self) -> &Rollup {
        Rollup::global()
    }
}

/// TODO
#[derive(Clone, Copy, Debug)]
pub struct LocalIntervalReport<'a> {
    pub(crate) interval: Duration,
    pub(crate) rollup: &'a Rollup,
}

impl<'a> LocalIntervalReport<'a> {
    /// TODO
    #[cfg(feature = "nightly")]
    pub const fn new(interval: Duration, rollup: &'a Rollup) -> Self {
        Self {
            interval,
            rollup,
        }
    }

    /// TODO
    #[cfg(not(feature = "nightly"))]
    pub fn new(interval: Duration, rollup: &'a Rollup) -> Self {
        Self {
            interval,
            rollup,
        }
    }
}

impl<'a> Reporter for LocalIntervalReport<'a> {
    #[inline]
    fn report(&self, thread_stats: &mut ThreadStats) {
        if let Some(lu) = thread_stats.last_updated() {
            let now = Instant::now();
            if now - lu > self.interval {
                thread_stats.flush_to(self.rollup());
                thread_stats.flush_to(Rollup::global());
                thread_stats.set_last_updated(now);
            }
        } else {
            thread_stats.set_last_updated(Instant::now());
        }
    }

    #[inline]
    fn rollup(&self) -> &Rollup {
        self.rollup
    }
}

/// TODO
#[derive(Clone, Copy, Debug, Default)]
pub struct AlwaysReport;

impl Reporter for AlwaysReport {
    #[inline(always)]
    fn report(&self, thread_stats: &mut ThreadStats) {
        thread_stats.flush_to(Rollup::global());
    }
}

/// TODO
#[derive(Clone, Copy, Debug)]
pub struct LocalAlwaysReport<'a>(pub(crate) &'a Rollup);

impl<'a> LocalAlwaysReport<'a> {
    /// TODO
    #[cfg(feature = "nightly")]
    pub const fn new(rollup: &'a Rollup) -> Self {
        LocalAlwaysReport(rollup)
    }

    /// TODO
    #[cfg(not(feature = "nightly"))]
    pub fn new(rollup: &'a Rollup) -> Self {
        LocalAlwaysReport(rollup)
    }
}

impl<'a> Reporter for LocalAlwaysReport<'a> {
    #[inline(always)]
    fn report(&self, thread_stats: &mut ThreadStats) {
        thread_stats.flush_to(self.0);
    }

    #[inline]
    fn rollup(&self) -> &Rollup {
        self.0
    }
}
