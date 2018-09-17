//! High-level interface to the DW1000
//!
//! This module implements a high-level interface to the DW1000. This is the
//! recommended way to access the DW1000 using this crate, unless you need the
//! greater flexibility provided by the register-level interface.


use hal::{
    prelude::*,
    gpio::{
        p0,
        Output,
        PushPull,
    },
    Spim,
};

use ll;


/// Entry point to the DW1000 driver API
pub struct DW1000<SPI>(ll::DW1000<SPI>);

impl<SPI> DW1000<SPI> where SPI: SpimExt {
    /// Create a new instance of `DW1000`
    ///
    /// Requires the SPI peripheral and the chip select pin that are connected
    /// to the DW1000.
    pub fn new(
        spim       : Spim<SPI>,
        chip_select: p0::P0_Pin<Output<PushPull>>
    )
        -> Self
    {
        DW1000(
            ll::DW1000::new(spim, chip_select)
        )
    }

    /// Provides direct access to the register-level API
    pub fn ll(&mut self) -> &mut ll::DW1000<SPI> {
        &mut self.0
    }
}
