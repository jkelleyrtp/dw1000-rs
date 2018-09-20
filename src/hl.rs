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
use nb;

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
    pub fn send_raw(&mut self, data: &[u8]) -> Result<(), Error> {
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

    /// Starts the receiver
    pub fn start_receiver(&mut self) -> Result<Receiver<SPI>, Error> {
        // For unknown reasons, the DW1000 get stuck in RX mode without ever
        // receiving anything, after receiving one good frame. Reset the
        // receiver to make sure its in a valid state before attempting to
        // receive anything.
        self.0
            .pmsc_ctrl0()
            .modify(|_, w|
                w.softreset(0b1110) // reset receiver
            )?;
        self.0
            .pmsc_ctrl0()
            .modify(|_, w|
                w.softreset(0b1111) // clear reset
            )?;

        // Set PLLLDT bit in EC_CTRL. According to the documentation of the
        // CLKPLL_LL bit in SYS_STATUS, this bit needs to be set to ensure the
        // reliable operation of the CLKPLL_LL bit. Since I've seen that bit
        // being set, I want to make sure I'm not just seeing crap.
        self.0
            .ec_ctrl()
            .modify(|_, w|
                w.pllldt(0b1)
            )?;

        // Now that PLLLDT is set, clear all bits in SYS_STATUS that depend on
        // it for reliable operation. After that is done, these bits should work
        // reliably.
        self.0
            .sys_status()
            .write(|w|
                w
                    .cplock(0b1)
                    .clkpll_ll(0b1)
            )?;

        // If we cared about MAC addresses, which we don't in this example, we'd
        // have to enable frame filtering at this point. By default it's off,
        // meaning we'll receive everything, no matter who it is addressed to.

        // We're expecting a preamble length of `64`. Set PAC size to the
        // recommended value for that preamble length, according to section
        // 4.1.1. The value we're writing to DRX_TUNE2 here also depends on the
        // PRF, which we expect to be 16 MHz.
        self.0
            .drx_tune2()
            .write(|w|
                // PAC size 8, with 16 MHz PRF
                w.value(0x311A002D)
            )?;

        // If we were going to receive at 110 kbps, we'd need to set the RXM110K
        // bit in the System Configuration register. We're expecting to receive
        // at 850 kbps though, so the default is fine. See section 4.1.3 for a
        // detailed explanation.

        self.0
            .sys_ctrl()
            .modify(|_, w|
                w.rxenab(0b1)
            )?;

        Ok(Receiver(&mut self.0))
    }
}


/// Receives data
///
/// Call [`DW1000::start_receiver`] to get an instance of `Receiver`.
pub struct Receiver<'r, SPI: 'r>(&'r mut ll::DW1000<SPI>);

impl<'r, SPI> Receiver<'r, SPI> where SPI: SpimExt {
    /// Receive data
    ///
    /// On success, it writes the received data into the buffer and returns the
    /// length of the received data.
    pub fn receive(&mut self, buffer: &mut [u8]) -> nb::Result<usize, Error> {
        let sys_status = self.0
            .sys_status()
            .read()
            .map_err(|error| Error::Spi(error))?;

        // Check for errors
        if sys_status.rxfce() == 0b1 {
            return Err(nb::Error::Other(Error::Fcs));
        }
        if sys_status.rxphe() == 0b1 {
            return Err(nb::Error::Other(Error::Phy));
        }

        // Is a frame ready?
        if sys_status.rxdfr() == 0b0 {
            // No frame ready
            return Err(nb::Error::WouldBlock);
        }

        // Read received frame
        let rx_finfo = self.0
            .rx_finfo()
            .read()
            .map_err(|error| Error::Spi(error))?;
        let rx_buffer = self.0
            .rx_buffer()
            .read()
            .map_err(|error| Error::Spi(error))?;

        let len = rx_finfo.rxflen() as usize;

        if buffer.len() < len {
            return Err(nb::Error::Other(
                Error::BufferTooSmall { required_len: len }
            ))
        }

        buffer[..len].copy_from_slice(&rx_buffer.data()[..len]);

        Ok(len)
    }
}


/// An error that can occur when sending or receiving data
#[derive(Debug)]
pub enum Error {
    /// Error occured while using SPI bus
    Spi(spim::Error),

    /// Receiver FCS error
    Fcs,

    /// PHY header error
    Phy,

    /// Buffer too small
    BufferTooSmall {
        /// Indicates how large a buffer would have been required
        required_len: usize,
    }
}

impl From<spim::Error> for Error {
    fn from(error: spim::Error) -> Self {
        Error::Spi(error)
    }
}

impl From<Error> for nb::Error<Error> {
    fn from(error: Error) -> Self {
        nb::Error::Other(error.into())
    }
}
