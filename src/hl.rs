//! High-level interface to the DW1000
//!
//! This module implements a high-level interface to the DW1000. This is the
//! recommended way to access the DW1000 using this crate, unless you need the
//! greater flexibility provided by the register-level interface.


use core::num::Wrapping;

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
use mac;


/// Entry point to the DW1000 driver API
pub struct DW1000<SPI> {
    ll:  ll::DW1000<SPI>,
    seq: Wrapping<u8>,
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
            ll:  ll::DW1000::new(spim, chip_select),
            seq: Wrapping(0),
        }
    }

    /// Provides direct access to the register-level API
    pub fn ll(&mut self) -> &mut ll::DW1000<SPI> {
        &mut self.ll
    }

    /// Sets the network id and address used for sending and receiving
    pub fn set_address(&mut self, address: mac::Address)
        -> Result<(), Error>
    {
        self.ll
            .panadr()
            .write(|w|
                w
                    .pan_id(address.pan_id)
                    .short_addr(address.short_addr)
            )?;

        Ok(())
    }

    /// Returns the network id and address used for sending and receiving
    pub fn get_address(&mut self) -> Result<mac::Address, Error> {
        let panadr = self.ll.panadr().read()?;

        Ok(mac::Address {
            pan_id:     panadr.pan_id(),
            short_addr: panadr.short_addr(),
        })
    }

    /// Broadcast raw data
    ///
    /// Broadcasts data without any MAC header.
    pub fn send(&mut self, data: &[u8]) -> Result<TxFuture<SPI>, Error> {
        // Sometimes, for unknown reasons, the DW1000 gets stuck in RX mode.
        // Starting the transmitter won't get it to enter TX mode, which means
        // all subsequent send operations will fail. Let's disable the
        // transceiver and force the chip into IDLE mode to make sure that
        // doesn't happen.
        self.force_idle()?;

        let seq = self.seq.0;
        self.seq += Wrapping(1);

        let header = mac::Header {
            frame_type:      mac::FrameType::Data,
            security:        mac::Security::None,
            frame_pending:   false,
            ack_request:     false,
            pan_id_compress: mac::PanIdCompress::Disabled,
            destination:     mac::Address::broadcast(),
            source:          self.get_address()?,
            seq:             seq,
        };

        // Prepare transmitter
        let mut len = 0;
        self.ll
            .tx_buffer()
            .write(|w| {
                // Write header
                len += header.write(&mut w.data()[len..]);

                // Write payload
                w.data()[len .. len+data.len()].copy_from_slice(data);
                len += data.len();

                // 2-byte CRC checksum is added automatically

                w
            })?;
        self.ll
            .tx_fctrl()
            .write(|w| {
                let tflen = len as u8 + 2;
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
        self.ll
            .sys_ctrl()
            .modify(|_, w|
                w
                    .txstrt(1)
            )?;

        Ok(TxFuture(&mut self.ll))
    }

    /// Starts the receiver
    pub fn receive(&mut self) -> Result<RxFuture<SPI>, Error> {
        // For unknown reasons, the DW1000 gets stuck in RX mode without ever
        // receiving anything, after receiving one good frame. Reset the
        // receiver to make sure its in a valid state before attempting to
        // receive anything.
        self.ll
            .pmsc_ctrl0()
            .modify(|_, w|
                w.softreset(0b1110) // reset receiver
            )?;
        self.ll
            .pmsc_ctrl0()
            .modify(|_, w|
                w.softreset(0b1111) // clear reset
            )?;

        // Enable frame filtering
        self.ll
            .sys_cfg()
            .modify(|_, w|
                w
                    .ffen(0b1) // enable frame filtering
                    .ffab(0b1) // receive beacon frames
                    .ffad(0b1) // receive data frames
                    .ffaa(0b1) // receive acknowledgement frames
                    .ffam(0b1) // receive MAC command frames
            )?;

        // Set PLLLDT bit in EC_CTRL. According to the documentation of the
        // CLKPLL_LL bit in SYS_STATUS, this bit needs to be set to ensure the
        // reliable operation of the CLKPLL_LL bit. Since I've seen that bit
        // being set, I want to make sure I'm not just seeing crap.
        self.ll
            .ec_ctrl()
            .modify(|_, w|
                w.pllldt(0b1)
            )?;

        // Now that PLLLDT is set, clear all bits in SYS_STATUS that depend on
        // it for reliable operation. After that is done, these bits should work
        // reliably.
        self.ll
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
        self.ll
            .drx_tune2()
            .write(|w|
                // PAC size 8, with 16 MHz PRF
                w.value(0x311A002D)
            )?;

        // If we were going to receive at 110 kbps, we'd need to set the RXM110K
        // bit in the System Configuration register. We're expecting to receive
        // at 850 kbps though, so the default is fine. See section 4.1.3 for a
        // detailed explanation.

        self.ll
            .sys_ctrl()
            .modify(|_, w|
                w.rxenab(0b1)
            )?;

        Ok(RxFuture(&mut self.ll))
    }


    /// Force the DW1000 into IDLE mode
    ///
    /// Any ongoing RX/TX operations will be aborted.
    pub fn force_idle(&mut self) -> Result<(), Error> {
        self.ll.sys_ctrl().write(|w| w.trxoff(0b1))?;
        while self.ll.sys_ctrl().read()?.trxoff() == 0b1 {}

        Ok(())
    }
}


/// Represents a TX operation that might not have completed
pub struct TxFuture<'r, SPI: 'r>(&'r mut ll::DW1000<SPI>);

impl<'r, SPI> TxFuture<'r, SPI> where SPI: SpimExt {
    /// Wait for the data to be sent
    pub fn wait(&mut self) -> nb::Result<(), Error> {
        let sys_status = self.0
            .sys_status()
            .read()
            .map_err(|error| Error::Spi(error))?;

        // Has the frame been sent?
        if sys_status.txfrs() == 0b0 {
            // Frame has not been sent
            return Err(nb::Error::WouldBlock);
        }

        // Frame sent. Reset all progress flags.
        self.0
            .sys_status()
            .write(|w|
                w
                    .txfrb(0b1) // Transmit Frame Begins
                    .txprs(0b1) // Transmit Preamble Sent
                    .txphs(0b1) // Transmit PHY Header Sent
                    .txfrs(0b1) // Transmit Frame Sent
            )
            .map_err(|error| Error::Spi(error))?;

        Ok(())
    }
}


/// Represents an RX operation that might not have finished
pub struct RxFuture<'r, SPI: 'r>(&'r mut ll::DW1000<SPI>);

impl<'r, SPI> RxFuture<'r, SPI> where SPI: SpimExt {
    /// Wait for data to be available
    pub fn wait(&mut self, buffer: &mut [u8]) -> nb::Result<usize, Error> {
        let sys_status = self.0
            .sys_status()
            .read()
            .map_err(|error| Error::Spi(error))?;

        // Is a frame ready?
        if sys_status.rxdfr() == 0b0 {
            // No frame ready. Check for errors.
            if sys_status.rxfce() == 0b1 {
                return Err(nb::Error::Other(Error::Fcs));
            }
            if sys_status.rxphe() == 0b1 {
                return Err(nb::Error::Other(Error::Phy));
            }
            if sys_status.rxrfsl() == 0b1 {
                return Err(nb::Error::Other(Error::ReedSolomon));
            }
            if sys_status.rxrfto() == 0b1 {
                return Err(nb::Error::Other(Error::FrameWaitTimeout));
            }
            if sys_status.rxovrr() == 0b1 {
                return Err(nb::Error::Other(Error::Overrun));
            }
            if sys_status.rxpto() == 0b1 {
                return Err(nb::Error::Other(Error::PreambleDetectionTimeout));
            }
            if sys_status.rxsfdto() == 0b1 {
                return Err(nb::Error::Other(Error::SfdTimeout));
            }
            // Some error flags that sound like valid errors aren't checked here,
            // because experience has shown that they seem to occur spuriously
            // without preventing a good frame from being received. Those are:
            // - LDEERR: Leading Edge Detection Processing Error
            // - RXPREJ: Receiver Preamble Rejection

            // No errors detected. That must mean the frame is just not ready
            // yet.
            return Err(nb::Error::WouldBlock);
        }

        // Reset status bits. This is not strictly necessary, but it helps, if
        // you have to inspect SYS_STATUS manually during debugging.
        self.0
            .sys_status()
            .write(|w|
                w
                    .rxprd(0b1)   // Receiver Preamble Detected
                    .rxsfdd(0b1)  // Receiver SFD Detected
                    .ldedone(0b1) // LDE Processing Done
                    .rxphd(0b1)   // Receiver PHY Header Detected
                    .rxphe(0b1)   // Receiver PHY Header Error
                    .rxdfr(0b1)   // Receiver Data Frame Ready
                    .rxfcg(0b1)   // Receiver FCS Good
                    .rxfce(0b1)   // Receiver FCS Error
                    .rxrfsl(0b1)  // Receiver Reed Solomon Frame Sync Loss
                    .rxrfto(0b1)  // Receiver Frame Wait Timeout
                    .ldeerr(0b1)  // Leading Edge Detection Processing Error
                    .rxovrr(0b1)  // Receiver Overrun
                    .rxpto(0b1)   // Preamble Detection Timeout
                    .rxsfdto(0b1) // Receiver SFD Timeout
                    .rxrscs(0b1)  // Receiver Reed-Solomon Correction Status
                    .rxprej(0b1)  // Receiver Preamble Rejection
            )
            .map_err(|error| Error::Spi(error))?;

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
    },

    /// Receiver Reed Solomon Frame Sync Loss
    ReedSolomon,

    /// Receiver Frame Wait Timeout
    FrameWaitTimeout,

    /// Receiver Overrun
    Overrun,

    /// Preamble Detection Timeout
    PreambleDetectionTimeout,

    /// Receiver SFD Timeout
    SfdTimeout,
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
