use std::ops;

/// Allocator statistics
#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
pub struct Summary {
    pub(super) allocations: usize,
    pub(super) deallocations: usize,
    pub(super) reallocations: usize,
    pub(super) bytes_allocated: usize,
    pub(super) bytes_deallocated: usize,
    pub(super) bytes_reallocated: isize,
}

impl Summary {
    /// Initializes an empty stats summary
    #[cfg(feature = "nightly")]
    pub const fn new() -> Self {
        Summary {
            allocations: 0,
            deallocations: 0,
            reallocations: 0,
            bytes_allocated: 0,
            bytes_deallocated: 0,
            bytes_reallocated: 0,
        }
    }

    /// Initializes an empty stats summary
    #[inline]
    #[cfg(not(feature = "nightly"))]
    pub fn new() -> Self {
        Summary {
            allocations: 0,
            deallocations: 0,
            reallocations: 0,
            bytes_allocated: 0,
            bytes_deallocated: 0,
            bytes_reallocated: 0,
        }
    }

    /// Zeroes out the data in this summary, making it equivalent to `Stats::new()`
    pub fn reset(&mut self) {
        *self = Self::new()
    }

    /// Returns the total quantity of bytes that have been allocated according to this statistical snapshot
    pub fn total_bytes_allocated(&self) -> usize {
        self.bytes_allocated
    }

    /// Returns the quantity of bytes that are still allocated according to this statistical snapshot
    pub fn outstanding_bytes_allocated(&self) -> isize {
        self.bytes_allocated.wrapping_sub(self.bytes_deallocated) as isize
    }
}

impl Summary {
    #[inline]
    pub(crate) fn allocate(&mut self, size: usize) {
        self.allocations += 1;
        self.bytes_allocated += size;
    }

    #[inline]
    pub(crate) fn deallocate(&mut self, size: usize) {
        self.deallocations += 1;
        self.bytes_deallocated += size;
    }

    #[inline]
    pub(crate) fn reallocate(&mut self, old_size: usize, new_size: usize) {
        self.reallocations += 1;
        let (change, net_deallocate) = new_size.overflowing_sub(old_size);
        if net_deallocate {
            debug_assert!(change + self.bytes_deallocated > self.bytes_deallocated);
            self.bytes_deallocated += change;
        } else {
            debug_assert!(change + self.bytes_allocated > self.bytes_allocated);
            self.bytes_allocated += change;
        }
        self.bytes_reallocated += change as isize;
    }
}

impl ops::Add for Summary {
    type Output = Summary;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a> ops::Add<&'a Summary> for Summary {
    type Output = Summary;

    fn add(mut self, rhs: &'a Summary) -> Self::Output {
        self += rhs;
        self
    }
}

impl ops::AddAssign for Summary {
    fn add_assign(&mut self, rhs: Self) {
        self.allocations += rhs.allocations;
        self.deallocations += rhs.deallocations;
        self.reallocations += rhs.reallocations;
        self.bytes_allocated += rhs.bytes_allocated;
        self.bytes_deallocated += rhs.bytes_deallocated;
        self.bytes_reallocated += rhs.bytes_reallocated;
    }
}

impl<'a> ops::AddAssign<&'a Summary> for Summary {
    fn add_assign(&mut self, rhs: &'a Summary) {
        self.allocations += rhs.allocations;
        self.deallocations += rhs.deallocations;
        self.reallocations += rhs.reallocations;
        self.bytes_allocated += rhs.bytes_allocated;
        self.bytes_deallocated += rhs.bytes_deallocated;
        self.bytes_reallocated += rhs.bytes_reallocated;
    }
}

impl ops::Sub for Summary {
    type Output = Summary;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl ops::SubAssign for Summary {
    fn sub_assign(&mut self, rhs: Self) {
        self.allocations -= rhs.allocations;
        self.deallocations -= rhs.deallocations;
        self.reallocations -= rhs.reallocations;
        self.bytes_allocated -= rhs.bytes_allocated;
        self.bytes_deallocated -= rhs.bytes_deallocated;
        self.bytes_reallocated -= rhs.bytes_reallocated;
    }
}
