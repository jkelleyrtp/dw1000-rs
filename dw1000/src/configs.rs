//! Configuration structs for sending and receiving
//!
//! This module houses the datastructures that control how frames are transmitted and received.
//! The configs are passed to the send and receive functions.

use crate::time::Duration;

/// Transmit configuration
pub struct TxConfig {
    /// Sets the bitrate of the transmission
    pub bitrate: BitRate,
    /// Sets the ranging bit in the transmitted frame.
    /// This has no effect on the capabilities of the DW1000
    pub ranging_enable: bool,
    /// Sets the PRF value of the transmission
    pub pulse_repetition_frequency: PulseRepetitionFrequency,
    /// The length of the preamble
    pub preamble_length: PreambleLength,
    /// The channel that the DW1000 will transmit at.
    pub channel: UwbChannel,
    /// The SFD sequence that is used to transmit a frame.
    pub sfd_sequence: SfdSequence,
}

impl Default for TxConfig {
    fn default() -> Self {
        TxConfig {
            bitrate: Default::default(),
            ranging_enable: false,
            pulse_repetition_frequency: Default::default(),
            preamble_length: Default::default(),
            channel: Default::default(),
            sfd_sequence: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// The bitrate at which a message is transmitted
pub enum BitRate {
    /// 110 kilobits per second.
    /// This is an unofficial extension from decawave.
    Kbps110 = 0b00,
    /// 850 kilobits per second.
    Kbps850 = 0b01,
    /// 6.8 megabits per second.
    Kbps6800 = 0b10,
}

impl Default for BitRate {
    fn default() -> Self {
        BitRate::Kbps6800
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// The PRF value
pub enum PulseRepetitionFrequency {
    /// 16 megahertz
    Mhz16 = 0b01,
    /// 64 megahertz
    Mhz64 = 0b10
}

impl Default for PulseRepetitionFrequency {
    fn default() -> Self {
        PulseRepetitionFrequency::Mhz16
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// An enum that specifies the length of the preamble.
///
/// Longer preambles improve the reception quality and thus range.
/// This comes at the cost of longer transmission times and thus power consumption and bandwidth use.
///
/// For the bit pattern, see table 16 in the user manual. Two bits TXPSR,then two bits PE.
pub enum PreambleLength {
    /// 64 bits of preamble.
    /// Only supported at Bitrate::Kbps6800.
    Bits64   = 0b0100,
    /// 128 bits of preamble.
    /// Only supported at Bitrate::Kbps850 & Bitrate::Kbps6800.
    /// Unofficial extension from decawave.
    Bits128  = 0b0101,
    /// 256 bits of preamble.
    /// Only supported at Bitrate::Kbps850 & Bitrate::Kbps6800.
    /// Unofficial extension from decawave.
    Bits256  = 0b0110,
    /// 512 bits of preamble.
    /// Only supported at Bitrate::Kbps850 & Bitrate::Kbps6800.
    /// Unofficial extension from decawave.
    Bits512  = 0b0111,
    /// 1024 bits of preamble.
    /// Only supported at Bitrate::Kbps850 & Bitrate::Kbps6800.
    Bits1024 = 0b1000,
    /// 1536 bits of preamble.
    /// Only supported at Bitrate::Kbps110.
    /// Unofficial extension from decawave.
    Bits1536 = 0b1001,
    /// 2048 bits of preamble.
    /// Only supported at Bitrate::Kbps110.
    /// Unofficial extension from decawave.
    Bits2048 = 0b1010,
    /// 4096 bits of preamble.
    /// Only supported at Bitrate::Kbps110.
    Bits4096 = 0b1100,
}

impl Default for PreambleLength {
    fn default() -> Self {
        PreambleLength::Bits128
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// An enum that allows the selection between different SFD sequences
pub enum SfdSequence {
    /// The standard sequence defined by the IEEE standard.
    IEEE,
    /// A sequence defined by Decawave that is supposed to be more robust.
    /// This is an unofficial addition.
    Decawave,
    /// Uses the sequence that is programmed in by the user.
    /// This is an unofficial addition.
    User
}

impl Default for SfdSequence {
    fn default() -> Self {
        SfdSequence::IEEE
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Receive configuration
pub struct RxConfig {
    /// Enable frame filtering
    ///
    /// If true, only frames directly addressed to this node and broadcasts will
    /// be received.
    ///
    /// Defaults to `true`.
    pub frame_filtering: bool,
    /// The expected preamble length.
    /// This affects the chosen PAC size.
    pub expected_preamble_length: PreambleLength,
    /// The bitrate that will be used for reception
    pub bitrate: BitRate,
    /// The type of SFD sequence that will be scanned for
    pub sfd_sequence: SfdSequence,
    /// The time after which reception is aborted when a preamble isn't detected.
    /// A value of 0 disables the timeout.
    /// If the value is greater than the maximum time, then the maximum time will be used.
    pub preamble_timeout: Duration,
    /// The channel that the DW1000 will listen at.
    pub channel: UwbChannel,
    /// Sets the PRF value of the reception
    pub pulse_repetition_frequency: PulseRepetitionFrequency,
}

impl Default for RxConfig {
    fn default() -> Self {
        Self {
            frame_filtering: true,
            expected_preamble_length: Default::default(),
            bitrate: Default::default(),
            sfd_sequence: Default::default(),
            preamble_timeout: Duration::from_nanos(0),
            channel: Default::default(),
            pulse_repetition_frequency: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// All the available UWB channels.
///
/// Note that while a channel may have more bandwidth than ~900 Mhz, the DW1000 can only send up to ~900 Mhz
pub enum UwbChannel {
    /// Channel 1
    /// - Center frequency: 3494.4 Mhz
    /// - Bandwidth: 499.2 Mhz
    Channel1 = 1,
    /// Channel 2
    /// - Center frequency: 3993.6 Mhz
    /// - Bandwidth: 499.2 Mhz
    Channel2 = 2,
    /// Channel 3
    /// - Center frequency: 4492.8 Mhz
    /// - Bandwidth: 499.2 Mhz
    Channel3 = 3,
    /// Channel 4
    /// - Center frequency: 3993.6 Mhz
    /// - Bandwidth: 1331.2 Mhz
    Channel4 = 4,
    /// Channel 5
    /// - Center frequency: 6489.6 Mhz
    /// - Bandwidth: 499.2 Mhz
    Channel5 = 5,
    /// Channel 7
    /// - Center frequency: 6489.6 Mhz
    /// - Bandwidth: 1081.6 Mhz
    Channel7 = 7,
}

impl Default for UwbChannel {
    fn default() -> Self {
        UwbChannel::Channel5
    }
}

impl UwbChannel {
    /// Gets the recommended preamble code
    pub fn get_recommended_preamble_code(&self, prf_value: PulseRepetitionFrequency) -> u8 {
        // Many have overlapping possibilities, so the numbers have been chosen so that there's no overlap here
        match (self, prf_value) {
            (UwbChannel::Channel1, PulseRepetitionFrequency::Mhz16) => 1,
            (UwbChannel::Channel2, PulseRepetitionFrequency::Mhz16) => 3,
            (UwbChannel::Channel3, PulseRepetitionFrequency::Mhz16) => 5,
            (UwbChannel::Channel4, PulseRepetitionFrequency::Mhz16) => 7,
            (UwbChannel::Channel5, PulseRepetitionFrequency::Mhz16) => 4,
            (UwbChannel::Channel7, PulseRepetitionFrequency::Mhz16) => 8,
            (UwbChannel::Channel1, PulseRepetitionFrequency::Mhz64) => 9,
            (UwbChannel::Channel2, PulseRepetitionFrequency::Mhz64) => 10,
            (UwbChannel::Channel3, PulseRepetitionFrequency::Mhz64) => 11,
            (UwbChannel::Channel4, PulseRepetitionFrequency::Mhz64) => 17,
            (UwbChannel::Channel5, PulseRepetitionFrequency::Mhz64) => 12,
            (UwbChannel::Channel7, PulseRepetitionFrequency::Mhz64) => 18,
        }
    }

    /// Gets the recommended value for the rf_txctrl register
    pub fn get_recommended_rf_txctrl(&self) -> u32 {
        // Values based on Table 38 of the DW1000 User Manual
        match self {
            UwbChannel::Channel1 => 0x00005C40,
            UwbChannel::Channel2 => 0x00045CA0,
            UwbChannel::Channel3 => 0x00086CC0,
            UwbChannel::Channel4 => 0x00045C80,
            UwbChannel::Channel5 => 0x001E3FE0,
            UwbChannel::Channel7 => 0x001E7DE0,
        }
    }

    /// Gets the recommended value for the tc_pgdelay register
    pub fn get_recommended_tc_pgdelay(&self) -> u8 {
        // Values based on Table 40 of the DW1000 User Manual
        match self {
            UwbChannel::Channel1 => 0xC9,
            UwbChannel::Channel2 => 0xC2,
            UwbChannel::Channel3 => 0xC5,
            UwbChannel::Channel4 => 0x95,
            UwbChannel::Channel5 => 0xC0,
            UwbChannel::Channel7 => 0x93,
        }
    }

    /// Gets the recommended value for the fs_pllcfg register
    pub fn get_recommended_fs_pllcfg(&self) -> u32 {
        // Values based on Table 43 of the DW1000 User Manual
        match self {
            UwbChannel::Channel1 => 0x09000407,
            UwbChannel::Channel2 | UwbChannel::Channel4 => 0x08400508,
            UwbChannel::Channel3 => 0x08401009,
            UwbChannel::Channel5 | UwbChannel::Channel7 => 0x0800041D,
        }
    }

    /// Gets the recommended value for the fs_plltune register
    pub fn get_recommended_fs_plltune(&self) -> u8 {
        // Values based on Table 44 of the DW1000 User Manual
        match self {
            UwbChannel::Channel1 => 0x1E,
            UwbChannel::Channel2 | UwbChannel::Channel4 => 0x26,
            UwbChannel::Channel3 => 0x56,
            UwbChannel::Channel5 | UwbChannel::Channel7 => 0xBE,
        }
    }
}