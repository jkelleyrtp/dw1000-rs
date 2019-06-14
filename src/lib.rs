//! Driver crate for the DW1000 UWB transceiver
//!
//! The recommended way to use this driver is the [high-level interface]. If you
//! require a higher degree of flexibility, you can use the
//! [register-level interface] instead.
//!
//! If you're using the DWM1001 module or DWM1001-Dev board, you probably don't
//! want to use this crate directly. Consider using the [`dwm1001`] crate
//! instead. The `dwm1001` crate also contains [usage examples] for this crate's
//! API.
//!
//! This driver is built on top of [`embedded-hal`], which means it is portable
//! and can be used on any platform that implements the `embedded-hal` API. It
//! is only well-tested on the Nordic nRF52832 microcontroller though (the
//! microcontroller used on the DWM1001 module), so be aware that you might run
//! into problems on other devices.
//!
//! [high-level interface]: hl/index.html
//! [register-level interface]: ll/index.html
//! [`dwm1001`]: https://crates.io/crates/dwm1001
//! [usage examples]: https://github.com/braun-robotics/rust-dwm1001/tree/master/examples
//! [`embedded-hal`]: https://crates.io/crates/embedded-hal


#![no_std]

#![deny(missing_docs)]


pub mod ll;
pub mod hl;
pub mod ranging;
pub mod time;


#[doc(no_inline)]
pub use ieee802154::mac;

pub use crate::hl::{
    DW1000,
    Error,
    Message,
    Ready,
    Receiving,
    Sending,
    Uninitialized,
};
