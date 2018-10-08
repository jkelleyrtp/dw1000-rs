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
pub struct DW1000<SPI, State> {
    ll:     ll::DW1000<SPI>,
    seq:    Wrapping<u8>,
    _state: State,
}

impl<SPI> DW1000<SPI, Uninitialized> where SPI: SpimExt {
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
            ll:     ll::DW1000::new(spim, chip_select),
            seq:    Wrapping(0),
            _state: Uninitialized,
        }
    }

    /// Initialize the DW1000
    pub fn init(self) -> DW1000<SPI, Ready> {
        // Nothing here yet.

        DW1000 {
            ll:     self.ll,
            seq:    self.seq,
            _state: Ready,
        }
    }
}

impl<SPI> DW1000<SPI, Ready> where SPI: SpimExt {
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
    pub fn send(&mut self,
        data:         &[u8],
        destination:  mac::Address,
        delayed_time: Option<u64>,
    )
        -> Result<TxFuture<SPI>, Error>
    {
        // Clear event counters
        self.ll.evc_ctrl().write(|w| w.evc_clr(0b1))?;
        while self.ll.evc_ctrl().read()?.evc_clr() == 0b1 {}

        // (Re-)Enable event counters
        self.ll.evc_ctrl().write(|w| w.evc_en(0b1))?;
        while self.ll.evc_ctrl().read()?.evc_en() == 0b1 {}

        // Sometimes, for unknown reasons, the DW1000 gets stuck in RX mode.
        // Starting the transmitter won't get it to enter TX mode, which means
        // all subsequent send operations will fail. Let's disable the
        // transceiver and force the chip into IDLE mode to make sure that
        // doesn't happen.
        self.force_idle()?;

        let seq = self.seq.0;
        self.seq += Wrapping(1);

        let frame = mac::Frame {
            header: mac::Header {
                frame_type:      mac::FrameType::Data,
                security:        mac::Security::None,
                frame_pending:   false,
                ack_request:     false,
                pan_id_compress: mac::PanIdCompress::Disabled,
                destination:     destination,
                source:          self.get_address()?,
                seq:             seq,
            },
            payload: data,
            footer: [0; 2],
        };

        delayed_time.map(|time| {
            self.ll
                .dx_time()
                .write(|w|
                    w.value(time)
                )
        });

        // Prepare transmitter
        let mut len = 0;
        self.ll
            .tx_buffer()
            .write(|w| {
                len += frame.write(&mut w.data(), mac::WriteFooter::No);
                w
            })?;
        self.ll
            .tx_fctrl()
            .modify(|_, w| {
                let tflen = len as u8 + 2;
                w
                    .tflen(tflen) // data length + two-octet CRC
                    .tfle(0)      // no non-standard length extension
                    .txboffs(0)   // no offset in TX_BUFFER
            })?;

        // Start transmission
        self.ll
            .sys_ctrl()
            .modify(|_, w|
                if delayed_time.is_some() { w.txdlys(0b1) } else { w }
                    .txstrt(0b1)
            )?;

        Ok(TxFuture(self))
    }

    /// Attempt to receive a frame
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

        // We're already resetting the receiver in the previous step, and that's
        // good enough to make my example program that's both sending and
        // receiving work very reliably over many hours (that's not to say it
        // comes unreliable after those hours, that's just when my test
        // stopped). However, I've seen problems with an example program that
        // only received, never sent, data. That got itself into some weird
        // state where it couldn't receive anymore.
        // I suspect that's because that example didn't have the following line
        // of code, while the send/receive example had that line of code, being
        // called from `send`.
        // While I haven't, as of this writing, run any hours-long tests to
        // confirm this does indeed fix the receive-only example, it seems
        // (based on my eyeball-only measurements) that the RX/TX example is
        // dropping fewer frames now.
        self.force_idle()?;

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

        Ok(RxFuture(self))
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

impl<SPI, State> DW1000<SPI, State> where SPI: SpimExt {
    /// Provides direct access to the register-level API
    pub fn ll(&mut self) -> &mut ll::DW1000<SPI> {
        &mut self.ll
    }
}


/// Represents a TX operation that might not have completed
pub struct TxFuture<'r, SPI: 'r>(&'r mut DW1000<SPI, Ready>);

impl<'r, SPI> TxFuture<'r, SPI> where SPI: SpimExt {
    /// Wait for the data to be sent
    pub fn wait(&mut self) -> nb::Result<(), Error> {
        // Check Half Period Warning Counter. If this is a delayed transmission,
        // this will indicate that the delay was too short, and the frame was
        // sent too late.
        let evc_hpw = self.0.ll()
            .evc_hpw()
            .read()
            .map_err(|error| Error::Spi(error))?
            .value();
        if evc_hpw != 0 {
            return Err(nb::Error::Other(Error::DelayedSendTooLate));
        }

        // Check Transmitter Power-Up Warning Counter. If this is a delayed
        // transmission, this indicates that the transmitter was still powering
        // up while sending, and the frame preamble might not have transmit
        // correctly.
        let evc_tpw = self.0.ll()
            .evc_tpw()
            .read()
            .map_err(|error| Error::Spi(error))?
            .value();
        if evc_tpw != 0 {
            return Err(nb::Error::Other(Error::DelayedSendPowerUpWarning));
        }

        let sys_status = self.0.ll()
            .sys_status()
            .read()
            .map_err(|error| Error::Spi(error))?;

        // Has the frame been sent?
        if sys_status.txfrs() == 0b0 {
            // Frame has not been sent
            return Err(nb::Error::WouldBlock);
        }

        // Frame sent. Reset all progress flags.
        self.0.ll()
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
pub struct RxFuture<'r, SPI: 'r>(&'r mut DW1000<SPI, Ready>);

impl<'r, SPI> RxFuture<'r, SPI> where SPI: SpimExt {
    /// Wait for data to be available
    pub fn wait<'b>(&mut self, buffer: &'b mut [u8])
        -> nb::Result<mac::Frame<'b>, Error>
    {
        let sys_status = self.0.ll()
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
        self.0.ll()
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
        let rx_finfo = self.0.ll()
            .rx_finfo()
            .read()
            .map_err(|error| Error::Spi(error))?;
        let rx_buffer = self.0.ll()
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

        let frame = mac::Frame::read(&buffer[..len])
            .map_err(|error| Error::Frame(error))?;

        Ok(frame)
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

    /// Frame could not be decoded
    Frame(mac::ReadError),

    /// A delayed frame could not be sent in time
    ///
    /// Please note that the frame was still sent. Replies could still arrive,
    /// and if it was a ranging frame, the resulting range measurement will be
    /// wrong.
    DelayedSendTooLate,

    /// Transmitter could not power up in time for delayed send
    ///
    /// The frame was still transmitted, but the first bytes of the preamble
    /// were likely corrupted.
    DelayedSendPowerUpWarning,
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



/// Indicates that the `DW1000` instance is not initialized yet
pub struct Uninitialized;

/// Indicates that the `DW1000` instance is ready to be used
pub struct Ready;
