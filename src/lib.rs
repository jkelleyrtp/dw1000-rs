//! Driver crate for the DW1000 UWB transceiver


#![no_std]

#![deny(missing_docs)]
#![deny(warnings)]


extern crate ieee802154;
extern crate nb;
extern crate serde;
extern crate serde_derive;
extern crate ssmarshal;

pub extern crate nrf52832_hal as hal;


pub mod ll;
pub mod hl;
pub mod util;


pub use ieee802154::mac;

pub use hl::{
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
