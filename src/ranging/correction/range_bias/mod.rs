//! Correction of ranging factors based on the following factors:
//!
//! * Measured Range
//! * Channel
//! * PRF

use core::cmp::{min, max};

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

/// Correct a measurement based on range bias
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

    // TODO: make this unreachable unchecked, all tables include u16::max as
    // their upper limits
    unreachable!()
}
