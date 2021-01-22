//! Partial implementation of the range bias as described in APS011 1.1

use crate::configs::{PulseRepetitionFrequency, RxConfig, UwbChannel};

/// The range bias table for PRF 16Mhz and a bandwidth of 500Mhz.
///
/// The values are taken from APS011 1.1
///
/// The first index is at -93 RSL(dBm) and every next index is 2 dBm higher. This RSL may not be the same as the RSSI!
/// The output is the bias in centimeters.
const RANGE_BIAS_CORRECTION_PRF16_MHZ500: [f32; 17] = [
    11.0, 10.6, 9.7, 8.4, 6.5, 3.6, 0.0, -3.1, -5.9, -8.4, -10.9, -12.7, -14.3, -16.3, -17.9,
    -18.7, -19.8,
];

/// The range bias table for PRF 64Mhz and a bandwidth of 500Mhz.
///
/// The values are taken from APS011 1.1
///
/// The first index is at -93 RSL(dBm) and every next index is 2 dBm higher. This RSL may not be the same as the RSSI!
/// The output is the bias in centimeters.
const RANGE_BIAS_CORRECTION_PRF64_MHZ500: [f32; 17] = [
    8.1, 7.6, 7.1, 6.2, 4.9, 4.2, 3.5, 2.1, 0.0, -2.7, -5.1, -6.9, -8.2, -9.3, -10.0, -10.5, -11.0,
];

/// The range bias table for PRF 16Mhz and a bandwidth of 900Mhz.
///
/// The values are taken from APS011 1.1
///
/// The first index is at -95 RSL(dBm) and every next index is 2 dBm higher. This RSL may not be the same as the RSSI!
/// The output is the bias in centimeters.
const RANGE_BIAS_CORRECTION_PRF16_MHZ900: [f32; 18] = [
    39.4, 35.6, 33.9, 32.1, 29.4, 25.4, 21.0, 15.8, 9.7, 4.2, 0.0, -5.1, -9.5, -13.8, -17.6, -21.0,
    -24.4, -27.5,
];

/// The range bias table for PRF 64Mhz and a bandwidth of 900Mhz.
///
/// The values are taken from APS011 1.1
///
/// The first index is at -95 RSL(dBm) and every next index is 2 dBm higher. This RSL may not be the same as the RSSI!
/// The output is the bias in centimeters.
const RANGE_BIAS_CORRECTION_PRF64_MHZ900: [f32; 18] = [
    28.4, 26.4, 24.5, 23.3, 19.7, 17.5, 15.3, 12.7, 9.1, 4.9, 0.0, -5.8, -10.0, -15.0, -19.9,
    -23.5, -26.6, -29.5,
];

/// Get the range bias based on the rx rsl and the config the radio used to receive the message
pub fn get_range_bias_cm(rsl: f32, rx_config: &RxConfig) -> f32 {
    #[allow(unused_imports)]
    // Not used on x86, but used on mcu target due to f32 core lib sillyness.
    use micromath::F32Ext;

    // Determine the message characteristics
    let low_bandwidth = match rx_config.channel {
        UwbChannel::Channel7 | UwbChannel::Channel4 => false,
        _ => true,
    };
    let low_prf = match rx_config.pulse_repetition_frequency {
        PulseRepetitionFrequency::Mhz16 => true,
        PulseRepetitionFrequency::Mhz64 => false,
    };

    // Turn the characteristics
    let (table, zero_index_value) = match (low_prf, low_bandwidth) {
        (false, false) => (RANGE_BIAS_CORRECTION_PRF64_MHZ900.as_ref(), -95.0),
        (true, false) => (RANGE_BIAS_CORRECTION_PRF16_MHZ900.as_ref(), -95.0),
        (false, true) => (RANGE_BIAS_CORRECTION_PRF64_MHZ500.as_ref(), -93.0),
        (true, true) => (RANGE_BIAS_CORRECTION_PRF16_MHZ500.as_ref(), -93.0),
    };

    let index = (rsl - zero_index_value) / 2.0;

    if index <= 0.0 {
        *table.first().unwrap()
    } else if index >= (table.len() - 1) as f32 {
        *table.last().unwrap()
    } else {
        let lower_index = index as usize;
        let upper_index = lower_index + 1;

        let lower_value = table[lower_index];
        let upper_value = table[upper_index];

        upper_value * index.fract() + lower_value * (1.0 - index.fract())
    }
}

/// Tries to improve the rssi estimation with figure 22 from the user manual (2.18)
pub fn improve_rssi_estimation(original_rssi: f32, rx_config: &crate::configs::RxConfig) -> f32 {
    #[allow(unused_imports)]
    // Not used on x86, but used on mcu target due to f32 core lib sillyness.
    use micromath::F32Ext;

    // The rssi multipliers to get from the original rssi to the new estimated rssi.
    // The multiplier at index 0 is at -105 dBm and increases 5 dBm every step.
    const PRF16: [f32; 11] = [
        105.0 / 105.0,
        102.5 / 102.5,
        100.0 / 100.0,
        97.5 / 97.5,
        95.0 / 95.0,
        92.5 / 92.5,
        90.0 / 90.0,
        85.0 / 87.5,
        77.0 / 85.0,
        69.5 / 82.5,
        64.0 / 80.0,
    ];
    const PRF64: [f32; 12] = [
        105.0 / 105.0,
        102.5 / 102.5,
        100.0 / 100.0,
        97.5 / 97.5,
        95.0 / 95.0,
        92.5 / 92.5,
        90.0 / 90.0,
        86.5 / 87.5,
        83.5 / 85.0,
        80.0 / 82.5,
        72.5 / 80.0,
        60.0 / 77.5,
    ];
    const START: f32 = -105.0;
    const STEP: f32 = 2.5;

    let table = match rx_config.pulse_repetition_frequency {
        PulseRepetitionFrequency::Mhz16 => PRF16.as_ref(),
        PulseRepetitionFrequency::Mhz64 => PRF64.as_ref(),
    };

    let index = (original_rssi - START) / STEP;

    if index < 0.0 {
        original_rssi * table.first().unwrap()
    } else if index >= (table.len() - 1) as f32 {
        original_rssi * table.last().unwrap()
    } else {
        let lower_index = index as usize;
        let upper_index = lower_index + 1;

        let lower_multiplier = table[lower_index];
        let upper_multiplier = table[upper_index];

        let multiplier =
            upper_multiplier * index.fract() + lower_multiplier * (1.0 - index.fract());

        original_rssi * multiplier
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_bias_cm_exact() {
        let rx_config = RxConfig::default();

        for (index, rsl) in (-93..-61).step_by(2).map(|i| i as f32).enumerate() {
            assert_eq!(
                get_range_bias_cm(rsl, &rx_config),
                RANGE_BIAS_CORRECTION_PRF16_MHZ500[index]
            );
        }
    }

    #[test]
    fn range_bias_cm_floating() {
        let rx_config = RxConfig::default();

        for (index, rsl) in (-93..-61).step_by(2).map(|i| i as f32).enumerate() {
            assert_eq!(
                get_range_bias_cm(rsl + 1.0, &rx_config),
                (RANGE_BIAS_CORRECTION_PRF16_MHZ500[index]
                    + RANGE_BIAS_CORRECTION_PRF16_MHZ500[index + 1])
                    / 2.0
            );
        }
    }

    #[test]
    fn range_bias_cm_too_low_still_valid() {
        let rx_config = RxConfig::default();
        assert_eq!(
            get_range_bias_cm(-1000.0, &rx_config),
            *RANGE_BIAS_CORRECTION_PRF16_MHZ500.first().unwrap()
        );
    }

    #[test]
    fn range_bias_cm_too_high_still_valid() {
        let rx_config = RxConfig::default();
        assert_eq!(
            get_range_bias_cm(1000.0, &rx_config),
            *RANGE_BIAS_CORRECTION_PRF16_MHZ500.last().unwrap()
        );
    }

    #[test]
    fn improve_rssi_rough_correctness() {
        let rx_config = RxConfig::default();

        assert!(improve_rssi_estimation(-103.0, &rx_config) > -103.1);
        assert!(improve_rssi_estimation(-103.0, &rx_config) < -102.9);

        assert!(improve_rssi_estimation(-85.0, &rx_config) > -78.0);
        assert!(improve_rssi_estimation(-85.0, &rx_config) < -76.0);

        assert!(improve_rssi_estimation(-83.0, &rx_config) > -72.0);
        assert!(improve_rssi_estimation(-83.0, &rx_config) < -70.0);
    }

    #[test]
    fn improve_rssi_too_low_still_valid() {
        let rx_config = RxConfig::default();
        assert_eq!(
            improve_rssi_estimation(-1000.0, &rx_config),
            -1000.0
        );
    }

    #[test]
    fn improve_rssi_too_high_still_valid() {
        let rx_config = RxConfig::default();
        assert_eq!(
            improve_rssi_estimation(1000.0, &rx_config),
            800.0
        );
    }
}
