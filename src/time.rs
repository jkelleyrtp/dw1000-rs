//! Time-related types based on the DW1000's system time


use core::ops::Add;

use serde_derive::{
    Deserialize,
    Serialize,
};

use crate::TIME_MAX;


/// Represents an instant in time
///
/// Internally uses the same 40-bit timestamps that the DW1000 uses.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Instant(u64);

impl Instant {
    /// Creates a new instance of `Instant`
    ///
    /// The given value must fit in a 40-bit timestamp, so:
    /// 0 <= `value` <= 2^40 - 1
    ///
    /// Returns `Some(...)`, if `value` is within the valid range, `None` if it
    /// isn't.
    ///
    /// # Example
    ///
    /// ``` rust
    /// use dw1000::{
    ///     time::Instant,
    ///     TIME_MAX,
    /// };
    ///
    /// let valid_instant   = Instant::new(TIME_MAX);
    /// let invalid_instant = Instant::new(TIME_MAX + 1);
    ///
    /// assert!(valid_instant.is_some());
    /// assert!(invalid_instant.is_none());
    /// ```
    pub fn new(value: u64) -> Option<Self> {
        if value <= TIME_MAX {
            Some(Instant(value))
        }
        else {
            None
        }
    }

    /// Returns the raw 40-bit timestamp
    ///
    /// The returned value is guaranteed to be in the following range:
    /// 0 <= `value` <= 2^40 - 1
    pub fn value(&self) -> u64 {
        self.0
    }

    /// Returns the amount of time passed between the two `Instant`s
    ///
    /// Assumes that `&self` represents a later time than the argument
    /// `earlier`. Please make sure that this is the case, as this method has no
    /// way of knowing (DW1000 timestamps can overflow, so comparing the
    /// numerical value of the timestamp doesn't tell anything about order).
    ///
    /// # Example
    ///
    /// ``` rust
    /// use dw1000::{
    ///     time::Instant,
    ///     TIME_MAX,
    /// };
    ///
    /// // `unwrap`ing here is okay, since we're passing constants that we know
    /// // are in the valid range.
    /// let instant_1 = Instant::new(TIME_MAX - 50).unwrap();
    /// let instant_2 = Instant::new(TIME_MAX).unwrap();
    /// let instant_3 = Instant::new(49).unwrap();
    ///
    /// // Works as expected, if the later timestamp is larger than the earlier
    /// // one.
    /// let duration = instant_2.duration_since(instant_1);
    /// assert_eq!(duration.0, 50);
    ///
    /// // Still works as expected, if the later timestamp is the numerically
    /// // smaller value.
    /// let duration = instant_3.duration_since(instant_2);
    /// assert_eq!(duration.0, 50);
    /// ```
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        if self.value() >= earlier.value() {
            Duration(self.value() - earlier.value())
        }
        else {
            Duration(TIME_MAX - earlier.value() + self.value() + 1)
        }
    }
}

/// A duration between two DW1000 system time instants
///
/// DW1000 timestamps are 40-bit numbers. Creating a `Duration` with a value
/// larger than 2^40 - 1 can lead to undefined behavior.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Duration(pub u64);

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        // Both `Instant` and `Duration` contain 40-bit numbers, so this
        // addition should never overflow.
        Instant((self.0 + rhs.0) % (TIME_MAX + 1))
    }
}
