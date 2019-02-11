//! Driver crate for the DW1000 UWB transceiver

#![no_std]

#![deny(missing_docs)]


pub mod ll;
pub mod hl;
pub mod ranging;
pub mod util;

pub use ieee802154::mac;

pub use crate::hl::{
    Duration,
    DW1000,
    Error,
    Instant,
    Message,
    Ready,
    TxFuture,
    Uninitialized,
};


/// The maximum value of 40-bit system time stamps.
pub const TIME_MAX: u64 = 0xffffffffff;
