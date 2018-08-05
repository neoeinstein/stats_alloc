use spin::Mutex;
use summary::Summary;

pub(crate) static GLOBAL_ALLOC_STATS: Rollup = Rollup::new();

/// TODO
#[derive(Debug)]
pub struct Rollup(Mutex<Summary>);

impl Rollup {
    /// TODO
    #[cfg(feature = "nightly")]
    pub const fn new() -> Rollup {
        Rollup(Mutex::new(Summary::new()))
    }

    /// TODO
    #[cfg(not(feature = "nightly"))]
    pub fn new() -> Rollup {
        Rollup(Mutex::new(Summary::new()))
    }

    /// TODO
    #[inline]
    pub fn global() -> &'static Rollup {
        &GLOBAL_ALLOC_STATS
    }

    /// TODO
    pub fn merge(&self, stats: &Summary) {
        *self.0.lock() += stats;
    }

    /// TODO
    pub fn summary(&self) -> Summary {
        *self.0.lock()
    }
}

