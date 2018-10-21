//! Correction Factors

#[allow(dead_code)]
/// Correction factors, with (upper_bounds_in_cm, correction_factor_in_cm)
pub const CORRECTION_FACTORS: &[(u16, i16)] = &[
    (25, -17),
    (50, -15),
    (75, -14),
    (100, -12),
    (125, -11),
    (200, -10),
    (250, -9),
    (325, -8),
    (375, -7),
    (425, -6),
    (475, -5),
    (525, -4),
    (575, -3),
    (625, -2),
    (675, -1),
    (750, 0),
    (825, 1),
    (925, 2),
    (1100, 3),
    (1500, 4),
    (1975, 5),
    (2325, 6),
    (3050, 7),
    (65535, 8),
];
