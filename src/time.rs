//! Time-related types based on the DW1000's system time


use core::ops::Add;
use serde::{Serialize, Deserialize};


/// The maximum value of 40-bit system time stamps.
pub const TIME_MAX: u64 = 0xffffffffff;


/// Represents an instant in time
///
/// You can get the current DW1000 system time by calling [`DW1000::sys_time`].
///
/// Internally uses the same 40-bit timestamps that the DW1000 uses.
///
/// [`DW1000::sys_time`]: ../hl/struct.DW1000.html#method.sys_time
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
    /// use dw1000::time::{
    ///     TIME_MAX,
    ///     Instant,
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
    /// use dw1000::time::{
    ///     TIME_MAX,
    ///     Instant,
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
    /// assert_eq!(duration.value(), 50);
    ///
    /// // Still works as expected, if the later timestamp is the numerically
    /// // smaller value.
    /// let duration = instant_3.duration_since(instant_2);
    /// assert_eq!(duration.value(), 50);
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

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        // Both `Instant` and `Duration` are guaranteed to contain 40-bit
        // numbers, so this addition will never overflow.
        let value = (self.value() + rhs.value()) % (TIME_MAX + 1);

        // We made sure to keep the result of the addition within `TIME_MAX`, so
        // the following will never panic.
        Instant::new(value).unwrap()
    }
}


/// A duration between two instants in DW1000 system time
///
/// Internally uses the same 40-bit timestamps that the DW1000 uses.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Duration(u64);

impl Duration {
    /// Creates a new instance of `Duration`
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
    /// use dw1000::time::{
    ///     TIME_MAX,
    ///     Duration,
    /// };
    ///
    /// let valid_duration   = Duration::new(TIME_MAX);
    /// let invalid_duration = Duration::new(TIME_MAX + 1);
    ///
    /// assert!(valid_duration.is_some());
    /// assert!(invalid_duration.is_none());
    /// ```
    pub fn new(value: u64) -> Option<Self> {
        if value <= TIME_MAX {
            Some(Duration(value))
        }
        else {
            None
        }
    }

    /// Creates an instance of `Duration` from a number of nanoseconds
    pub fn from_nanos(nanos: u32) -> Self {
        // `nanos` takes up at most 32 bits before it is cast to `u64`. That
        // means the result of the multiplication fits within 38 bits, so the
        // following should never panic.
        Duration::new(nanos as u64 * 64).unwrap()
    }

    /// Returns the raw 40-bit timestamp
    ///
    /// The returned value is guaranteed to be in the following range:
    /// 0 <= `value` <= 2^40 - 1
    pub fn value(&self) -> u64 {
        self.0
    }
}
