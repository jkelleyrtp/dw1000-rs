//! Contains utility functions that are useful when working with the DW1000


use TIME_MAX;


/// Determines the duration between to time stamps
///
/// Expects two 40-bit system time stamps and returns the duration between the
/// two, taking potential overflow into account.
///
/// # Panics
///
/// Panics, if the time stamps passed don't fit within 40 bits.
pub fn duration_between(earlier: u64, later: u64) -> u64 {
    assert!(earlier <= TIME_MAX);
    assert!(later   <= TIME_MAX);

    if later >= earlier {
        later - earlier
    }
    else {
        TIME_MAX - earlier + later + 1
    }
}
