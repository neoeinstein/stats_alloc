use std::{
    alloc::Layout,
    cell::RefCell,
    fmt,
    time::Instant,
};
use accum::{
    reporter::Reporter,
    rollup::Rollup,
};
use summary::Summary;

thread_local! {
    static THREAD_ALLOC_STATS: RefCell<ThreadStats> = RefCell::default();
}

/// TODO
#[derive(Default)]
pub struct ThreadStats {
    stats: Summary,
    last_updated: Option<Instant>,
    thread_total: Summary,
    reporter: Option<&'static (dyn Reporter + 'static)>,
}

impl ThreadStats {
    pub(super) fn set_last_updated(&mut self, now: Instant) {
        self.last_updated = Some(now);
    }

    pub(super) fn last_updated(&self) -> Option<Instant> {
        self.last_updated
    }

    #[inline]
    pub(super) fn flush_to(&mut self, rollup: &Rollup) {
        rollup.merge(&self.stats);
        self.thread_total += self.stats;
        self.stats.reset();
    }

    /// TODO
    pub fn set_thread_reporter(reporter: &'static (dyn Reporter + 'static)) {
        Self::flush();
        THREAD_ALLOC_STATS.with(|s: &RefCell<ThreadStats>| {
            s.borrow_mut().reporter = Some(reporter);
        })
    }

    /// TODO
    pub fn flush() {
        THREAD_ALLOC_STATS.with(|s: &RefCell<ThreadStats>| {
            let thread_stats = &mut *s.borrow_mut();
            let now = Instant::now();
            if let Some(reporter) = thread_stats.reporter {
                thread_stats.flush_to(reporter.rollup());
            } else {
                thread_stats.flush_to(Rollup::global());
            }
            thread_stats.set_last_updated(now);
        });
    }

    /// TODO
    pub fn summary() -> Summary {
        THREAD_ALLOC_STATS.with(|s| {
            let data = s.borrow();
            data.stats + data.thread_total
        })
    }

    fn report<T: Reporter>(&mut self, default_reporter: &T) {
        if let Some(r) = self.reporter {
            r.report(self);
        } else {
            default_reporter.report(self);
        }
    }

    #[inline]
    pub(crate) fn alloc<T: Reporter>(layout: Layout, default_reporter: &T) {
        let _ = THREAD_ALLOC_STATS.try_with(|s: &RefCell<ThreadStats>| {
            let data = &mut s.borrow_mut();
            data.stats.allocate(layout.size());
            data.report(default_reporter);
        });
    }

    #[inline]
    pub(crate) fn dealloc<T: Reporter>(layout: Layout, default_reporter: &T) {
        let _ = THREAD_ALLOC_STATS.try_with(|s: &RefCell<ThreadStats>| {
            let data = &mut s.borrow_mut();
            data.stats.deallocate(layout.size());
            data.report(default_reporter);
        });
    }

    #[inline]
    pub(crate) fn realloc<T: Reporter>(existing: Layout, new_size: usize, default_reporter: &T) {
        let _ = THREAD_ALLOC_STATS.try_with(|s: &RefCell<ThreadStats>| {
            let data = &mut s.borrow_mut();
            data.stats.reallocate(existing.size(), new_size);
            data.report(default_reporter);
        });
    }
}

impl Drop for ThreadStats {
    /// TODO
    #[inline]
    fn drop(&mut self) {
        if let Some(reporter) = self.reporter {
            self.flush_to(reporter.rollup());
        } else {
            self.flush_to(Rollup::global());
        }
    }
}

impl fmt::Debug for ThreadStats {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ThreadStats")
            .field("stats", &self.stats)
            .field("thread_total", &self.thread_total)
            .field("last_updated", &self.last_updated)
            .field("reporter", &if self.reporter.is_some() { Some("reporter") } else { None })
            .finish()
    }
}

