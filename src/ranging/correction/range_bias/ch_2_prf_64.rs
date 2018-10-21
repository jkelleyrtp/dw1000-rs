//! Correction Factors

#[allow(dead_code)]
/// Correction factors, with (upper_bounds_in_cm, correction_factor_in_cm)
pub const CORRECTION_FACTORS: &[(u16, i16)] = &[
    (25, -17),
    (50, -16),
    (75, -14),
    (100, -13),
    (150, -11),
    (225, -10),
    (300, -9),
    (350, -8),
    (425, -7),
    (475, -6),
    (525, -5),
    (600, -4),
    (650, -3),
    (700, -2),
    (775, -1),
    (825, 0),
    (925, 1),
    (1050, 2),
    (1225, 3),
    (1700, 4),
    (2225, 5),
    (2625, 6),
    (3450, 7),
    (65535, 8),
];
