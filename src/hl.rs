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
    spim,
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

    /// Broadcast raw data
    ///
    /// Broadcasts data without any MAC header.
    pub fn send_raw(&mut self, data: &[u8]) -> Result<(), spim::Error> {
        // Prepare transmitter
        self.0
            .tx_buffer()
            .write(|w|
                w.data(data)
            )?;
        self.0
            .tx_fctrl()
            .write(|w| {
                let tflen = data.len() as u8 + 2;
                w
                    .tflen(tflen) // data length + two-octet CRC
                    .tfle(0)      // no non-standard length extension
                    .txbr(0b01)   // 850 kbps bit rate
                    .tr(0b0)      // no ranging
                    .txprf(0b01)  // pulse repetition frequency: 16 MHz
                    .txpsr(0b01)  // preamble length: 64
                    .pe(0b00)     // no non-standard preamble length
                    .txboffs(0)   // no offset in TX_BUFFER
                    .ifsdelay(0)  // no delay between frames
            })?;

        // Start transmission
        self.0
            .sys_ctrl()
            .modify(|_, w|
                w
                    .txstrt(1)
            )?;

        // Wait until frame is sent
        loop {
            let sys_status = self.0
                .sys_status()
                .read()?;

            // Has the frame been sent?
            if sys_status.txfrs() == 0b1 {
                // Frame sent. Reset all progress flags.
                self.0
                    .sys_status()
                    .write(|w|
                        w
                            .txfrb(0b1) // Transmit Frame Begins
                            .txprs(0b1) // Transmit Preamble Sent
                            .txphs(0b1) // Transmit PHY Header Sent
                            .txfrs(0b1) // Transmit Frame Sent
                    )?;

                break;
            }
        }

        Ok(())
    }
}
