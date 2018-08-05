use accum::thread_local::ThreadStats;
use summary::Summary;

/// A snapshot of thread allocation statistics, which can be used to determine
/// allocation changes while the `Region` is alive.
///
/// This structure only looks at allocation changes on the current thread.
#[derive(Clone, Copy, Debug)]
pub struct Region {
    initial_stats: Summary,
}

impl Region {
    /// Creates a new region using statistics from the given instrumented
    /// allocator.
    #[inline]
    pub fn new() -> Self {
        Region {
            initial_stats: ThreadStats::summary(),
        }
    }

    /// Returns the statistics as of instantiation or the last reset.
    #[inline]
    pub fn initial(&self) -> Summary {
        self.initial_stats
    }

    /// Returns the difference between the currently reported statistics and
    /// those provided by `initial()`.
    #[inline]
    pub fn change(&self) -> Summary {
        ThreadStats::summary() - self.initial_stats
    }

    /// Returns the difference between the currently reported statistics and
    /// those provided by `initial()`, resetting initial to the latest
    /// reported statistics.
    #[inline]
    pub fn change_and_reset(&mut self) -> Summary {
        let latest = ThreadStats::summary();
        let diff = latest - self.initial_stats;
        self.initial_stats = latest;
        diff
    }

    /// Resets the initial initial to the latest reported statistics from the
    /// referenced allocator.
    #[inline]
    pub fn reset(&mut self) {
        self.initial_stats = ThreadStats::summary();
    }
}
