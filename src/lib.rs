//! Driver crate for the DW1000 UWB transceiver


#![no_std]

#![deny(missing_docs)]
#![deny(warnings)]


extern crate nrf52_hal;


use nrf52_hal::{
    prelude::*,
    gpio::{
        p0,
        Output,
        PushPull,
    },
    spim,
    Spim,
};


/// Entry point to the DW1000 driver API
pub struct DW1000<SPI> {
    spim       : Spim<SPI>,
    chip_select: p0::P0_Pin<Output<PushPull>>,
}

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
        DW1000 {
            spim,
            chip_select,
        }
    }

    /// Read the device identifier (DEV_ID)
    pub fn dev_id(&mut self) -> Result<DEV_ID, spim::Error> {
        let header =
            (0          & 0x80) |  // read
            (0          & 0x40) |  // no sub-index
            (DEV_ID::ID & 0x3f);   // index of DEV_ID register
        let tx_buffer = [header];

        let mut r = DEV_ID::new();

        self.spim.read(&mut self.chip_select, &tx_buffer, r.rx_buffer())?;

        Ok(r)
    }
}


/// Implemented for all registers
///
/// This trait is for internal use only. Users of this library should never need
/// to implement this trait, nor use its associated items.
///
/// The DW1000 user manual, section 7.1, specifies what the values of those
/// constant should be for each register.
pub trait Register {
    /// The register ID
    const ID:  u8;

    /// The lenght of the register
    const LEN: usize;

    /// Creates an instance of the register
    fn new() -> Self;

    /// Returns a mutable reference to the register's internal buffer
    ///
    /// SPI is a synchronous interface, which means a bytes is received for
    /// every byte that is sent, even though the bytes we receive while sending
    /// something end up being ignored. Still, we need room for those bytes in
    /// the buffer, so the length of the buffer must be equal to the length of
    /// the register plus the length of the transaction header.
    fn rx_buffer(&mut self) -> &mut [u8];
}


/// Device identifier - includes device type and revision info
#[allow(non_camel_case_types)]
pub struct DEV_ID([u8; 5]);

impl Register for DEV_ID {
    const ID:  u8    = 0x00;
    const LEN: usize = 4;

    fn new() -> Self {
        DEV_ID([0; 5])
    }

    fn rx_buffer(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl DEV_ID {
    /// Register Identification Tag
    pub fn ridtag(&self) -> u16 {
        ((self.0[4] as u16) << 8) | self.0[3] as u16
    }

    /// Model
    pub fn model(&self) -> u8 {
        self.0[2]
    }

    /// Version
    pub fn ver(&self) -> u8 {
        (self.0[1] & 0xf0) >> 4
    }

    /// Revision
    pub fn rev(&self) -> u8 {
        self.0[1] & 0x0f
    }
}


#[cfg(test)]
mod tests {
    use super::DEV_ID;


    #[test]
    fn dev_id_should_provide_access_to_its_fields() {
        let dev_id = DEV_ID([0x00, 0x30, 0x01, 0xca, 0xde]);

        assert_eq!(dev_id.rev()   , 0     );
        assert_eq!(dev_id.ver()   , 3     );
        assert_eq!(dev_id.model() , 1     );
        assert_eq!(dev_id.ridtag(), 0xDECA);
    }
}
