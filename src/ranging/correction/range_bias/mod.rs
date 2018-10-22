//! Correction of ranging factors based on the following factors:
//!
//! * Measured Range
//! * Channel
//! * PRF

use core::cmp::{min, max};
use core::hint::unreachable_unchecked;

// These are the constants
pub mod ch_1_prf_16;
pub mod ch_1_prf_64;
pub mod ch_2_prf_16;
pub mod ch_2_prf_64;
pub mod ch_3_prf_16;
pub mod ch_3_prf_64;
pub mod ch_4_prf_16;
pub mod ch_4_prf_64;
pub mod ch_5_prf_16;
pub mod ch_5_prf_64;
pub mod ch_7_prf_16;
pub mod ch_7_prf_64;

/// Type alias for correction Factors
pub type CorrectionFactor = [(u16, i16)];

/// Correct a measurement based on range bias.
///
/// NOTE: The Correction Factors used must cover the entire range of `measured_range_cm`, e.g.
/// the last item in the array must have an upper bound of <u16>::max_value(), otherwise this
/// function may exhibit undefined behavior
pub fn correct_range_bias(
    correction: &'static CorrectionFactor,
    measured_range_cm: u16
) -> u16 {
    for (dist, correction) in correction {
        if measured_range_cm <= *dist {
            let val = measured_range_cm as i32 + *correction as i32;

            // Clamp to sane u16 values
            return max(0, min(val, <u16>::max_value() as i32)) as u16;
        }
    }

    // NOTE: This is sound if and only if the upper bounds of the correction factor cover the
    // full u16 range. In this case, it is not possible for the loop not to have already returned
    // by this point
    unsafe {
        unreachable_unchecked();
    }
}
