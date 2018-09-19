//! Driver crate for the DW1000 UWB transceiver


#![no_std]

#![deny(missing_docs)]
#![deny(warnings)]


extern crate nrf52832_hal as hal;


pub mod ll;
pub mod hl;


pub use hl::{
    DW1000,
    Error,
};
