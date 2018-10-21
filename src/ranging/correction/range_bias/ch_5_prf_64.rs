//! Correction Factors

#[allow(dead_code)]
/// Correction factors, with (upper_bounds_in_cm, correction_factor_in_cm)
pub const CORRECTION_FACTORS: &[(u16, i16)] = &[
    (25, -17),
    (50, -14),
    (75, -12),
    (100, -11),
    (150, -10),
    (175, -9),
    (225, -8),
    (250, -7),
    (300, -6),
    (325, -5),
    (375, -4),
    (400, -3),
    (425, -2),
    (475, -1),
    (525, 0),
    (575, 1),
    (650, 2),
    (750, 3),
    (1050, 4),
    (1375, 5),
    (1625, 6),
    (2125, 7),
    (65535, 8),
];
