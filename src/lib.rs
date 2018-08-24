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
        // Set up the transmit buffer
        //
        // It consists of only one byte for the transaction header. Since this
        // is a read operation, there is no transaction body.
        //
        // The transaction signals a read without a sub-index, which means it's
        // one byte long. This byte consists of the following bits:
        //   7: 0 for read
        //   6: 0 for no sub-index
        // 5-0: 0 for DEV_ID register
        let tx_buffer = [0u8];

        // Set up the receive buffer
        //
        // SPI is a synchronous interface, so we're going to receive a byte for
        // every one we send. That means in addition to the 4 bytes we actually
        // expect, we need an additional one that we receive while we send the
        // header.
        let mut rx_buffer = [0u8; 5];

        self.spim.read(&mut self.chip_select, &tx_buffer, &mut rx_buffer)?;

        Ok(DEV_ID(rx_buffer))
    }
}


/// The device identifier (DEV_ID)
#[allow(non_camel_case_types)]
pub struct DEV_ID([u8; 5]);

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
