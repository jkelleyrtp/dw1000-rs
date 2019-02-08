//! High-level interface to the DW1000
//!
//! This module implements a high-level interface to the DW1000. This is the
//! recommended way to access the DW1000 using this crate, unless you need the
//! greater flexibility provided by the register-level interface.


use core::{
    fmt,
    num::Wrapping,
    ops::Add,
};

use embedded_hal::{
    blocking::spi,
    digital::OutputPin,
};
use nb;
use serde_derive::{Serialize, Deserialize};
use ssmarshal;

use crate::ll;
use crate::mac;
use crate::TIME_MAX;


/// Entry point to the DW1000 driver API
pub struct DW1000<SPI, CS, State> {
    ll:     ll::DW1000<SPI, CS>,
    seq:    Wrapping<u8>,
    _state: State,
}

impl<SPI, CS> DW1000<SPI, CS, Uninitialized>
    where
        SPI: spi::Transfer<u8> + spi::Write<u8>,
        CS:  OutputPin,
{
    /// Create a new instance of `DW1000`
    ///
    /// Requires the SPI peripheral and the chip select pin that are connected
    /// to the DW1000.
    pub fn new(
        spi        : SPI,
        chip_select: CS,
    )
        -> Self
    {
        DW1000 {
            ll:     ll::DW1000::new(spi, chip_select),
            seq:    Wrapping(0),
            _state: Uninitialized,
        }
    }

    /// Initialize the DW1000
    ///
    /// The DW1000's default configuration is somewhat inconsistent, and the
    /// user manual (section 2.5.5) has a long list of default configuration
    /// values that should be changed to guarantee everything works correctly.
    /// This method does just that.
    ///
    /// Please note that this method assumes that you kept the default
    /// configuration. It is generally recommended not to change configuration
    /// before calling this method.
    pub fn init(mut self) -> Result<DW1000<SPI, CS, Ready>, Error<SPI>> {
        // Set AGC_TUNE1. See user manual, section 2.5.5.1.
        self.ll.agc_tune1().write(|w| w.value(0x8870))?;

        // Set AGC_TUNE2. See user manual, section 2.5.5.2.
        self.ll.agc_tune2().write(|w| w.value(0x2502A907))?;

        // Set DRX_TUNE2. See user manual, section 2.5.5.3.
        self.ll.drx_tune2().write(|w| w.value(0x311A002D))?;

        // Set NTM. See user manual, section 2.5.5.4. This improves performance
        // in line-of-sight conditions, but might not be the best choice if non-
        // line-of-sight performance is important.
        self.ll.lde_cfg1().modify(|_, w| w.ntm(0xD))?;

        // Set LDE_CFG2. See user manual, section 2.5.5.5.
        self.ll.lde_cfg2().write(|w| w.value(0x1607))?;

        // Set TX_POWER. See user manual, section 2.5.5.6.
        self.ll.tx_power().write(|w| w.value(0x0E082848))?;

        // Set RF_TXCTRL. See user manual, section 2.5.5.7.
        self.ll.rf_txctrl().modify(|_, w|
            w
                .txmtune(0b1111)
                .txmq(0b111)
        )?;

        // Set TC_PGDELAY. See user manual, section 2.5.5.8.
        self.ll.tc_pgdelay().write(|w| w.value(0xC0))?;

        // Set FS_PLLTUNE. See user manual, section 2.5.5.9.
        self.ll.fs_plltune().write(|w| w.value(0xBE))?;

        // Set LDELOAD. See user manual, section 2.5.5.10.
        self.ll.pmsc_ctrl0().modify(|_, w| w.sysclks(0b01))?;
        self.ll.otp_ctrl().modify(|_, w| w.ldeload(0b1))?;
        while self.ll.otp_ctrl().read()?.ldeload() == 0b1 {}
        self.ll.pmsc_ctrl0().modify(|_, w| w.sysclks(0b00))?;

        // Set LDOTUNE. See user manual, section 2.5.5.11.
        self.ll.otp_addr().write(|w| w.value(0x004))?;
        self.ll.otp_ctrl().modify(|_, w|
            w
                .otprden(0b1)
                .otpread(0b1)
        )?;
        while self.ll.otp_ctrl().read()?.otpread() == 0b1 {}
        let ldotune_low = self.ll.otp_rdat().read()?.value();
        if ldotune_low != 0 {
            self.ll.otp_addr().write(|w| w.value(0x005))?;
            self.ll.otp_ctrl().modify(|_, w|
                w
                    .otprden(0b1)
                    .otpread(0b1)
            )?;
            while self.ll.otp_ctrl().read()?.otpread() == 0b1 {}
            let ldotune_high = self.ll.otp_rdat().read()?.value();

            let ldotune = ldotune_low as u64 | (ldotune_high as u64) << 32;
            self.ll.ldotune().write(|w| w.value(ldotune))?;
        }

        Ok(DW1000 {
            ll:     self.ll,
            seq:    self.seq,
            _state: Ready,
        })
    }
}

impl<SPI, CS> DW1000<SPI, CS, Ready>
    where
        SPI: spi::Transfer<u8> + spi::Write<u8>,
        CS:  OutputPin,
{
    /// Sets the RX and TX antenna delays
    pub fn set_antenna_delay(&mut self, rx_delay: u16, tx_delay: u16)
        -> Result<(), Error<SPI>>
    {
        self.ll
            .lde_rxantd()
            .write(|w| w.value(rx_delay))?;
        self.ll
            .tx_antd()
            .write(|w| w.value(tx_delay))?;

        Ok(())
    }

    /// Returns the TX antenna delay
    pub fn get_tx_antenna_delay(&mut self)
        -> Result<Duration, Error<SPI>>
    {
        let tx_antenna_delay = self.ll.tx_antd().read()?.value();
        Ok(Duration(tx_antenna_delay as u64))
    }

    /// Sets the network id and address used for sending and receiving
    pub fn set_address(&mut self, address: mac::Address)
        -> Result<(), Error<SPI>>
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
    pub fn get_address(&mut self)
        -> Result<mac::Address, Error<SPI>>
    {
        let panadr = self.ll.panadr().read()?;

        Ok(mac::Address {
            pan_id:     panadr.pan_id(),
            short_addr: panadr.short_addr(),
        })
    }

    /// Converts a delay in nanoseconds into a future timestamp
    ///
    /// Takes a delay in nanoseconds and returns a timestamp in the future,
    /// based on the delay and the current system time. This time stamp can be
    /// used for a delayed transmission.
    ///
    /// The result will fit within 40 bits, which means it will always be a
    /// valid timer value.
    pub fn time_from_delay(&mut self, delay_ns: u32)
        -> Result<Instant, Error<SPI>>
    {
        let sys_time = self.ll.sys_time().read()?.value();

        // This should always be the case, unless we're getting crap back from
        // the lower-level layer.
        assert!(sys_time <= TIME_MAX);

        // All of the following operations should be safe against undefined
        // behavior. First, `delay_ns` fills 32 bits before it is cast to `u64`.
        // The resulting number fills at most 38 bits. `sys_time` fits within 40
        // bits (as we've verified above), so the result of the addition fits
        // within 41 bits.
        let delay   = delay_ns as u64 * 64;
        let tx_time = sys_time + delay;

        // Make sure our delayed time doesn't overflow the 40-bit timer.
        let tx_time = if tx_time > TIME_MAX {
            tx_time - TIME_MAX
        }
        else {
            tx_time
        };

        Ok(Instant(tx_time))
    }

    /// Broadcast raw data
    ///
    /// Broadcasts data without any MAC header.
    pub fn send(&mut self,
        data:         &[u8],
        destination:  mac::Address,
        delayed_time: Option<Instant>,
    )
        -> Result<TxFuture<SPI, CS>, Error<SPI>>
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
                    w.value(time.0)
                )
        });

        // Prepare transmitter
        let mut len = 0;
        self.ll
            .tx_buffer()
            .write(|w| {
                len += frame.encode(&mut w.data(), mac::WriteFooter::No);
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
    pub fn receive(&mut self)
        -> Result<RxFuture<SPI, CS>, Error<SPI>>
    {
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
    pub fn force_idle(&mut self)
        -> Result<(), Error<SPI>>
    {
        self.ll.sys_ctrl().write(|w| w.trxoff(0b1))?;
        while self.ll.sys_ctrl().read()?.trxoff() == 0b1 {}

        Ok(())
    }

    /// Clear all interrupt flags
    pub fn clear_interrupts(&mut self)
        -> Result<(), Error<SPI>>
    {
        self.ll.sys_mask().write(|w| w)?;
        Ok(())
    }
}

impl<SPI, CS, State> DW1000<SPI, CS, State> {
    /// Provides direct access to the register-level API
    pub fn ll(&mut self) -> &mut ll::DW1000<SPI, CS> {
        &mut self.ll
    }
}


/// Represents a TX operation that might not have completed
pub struct TxFuture<'r, SPI, CS>(&'r mut DW1000<SPI, CS, Ready>);

impl<'r, SPI, CS> TxFuture<'r, SPI, CS>
    where
        SPI: spi::Transfer<u8> + spi::Write<u8>,
        CS:  OutputPin,
{
    /// Wait for the data to be sent
    pub fn wait(&mut self)
        -> nb::Result<(), Error<SPI>>
    {
        // Check Half Period Warning Counter. If this is a delayed transmission,
        // this will indicate that the delay was too short, and the frame was
        // sent too late.
        let evc_hpw = self.0.ll()
            .evc_hpw()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?
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
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?
            .value();
        if evc_tpw != 0 {
            return Err(nb::Error::Other(Error::DelayedSendPowerUpWarning));
        }

        // ATTENTION:
        // If you're changing anything about which SYS_STATUS flags are being
        // checked in this method, also make sure to update `enable_interrupts`.
        let sys_status = self.0.ll()
            .sys_status()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;

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
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;

        Ok(())
    }

    /// Enables interrupts for the events that `wait` checks
    ///
    /// Overwrites any interrupt flags that were previously set.
    pub fn enable_interrupts(&mut self)
        -> Result<(), Error<SPI>>
    {
        self.0.ll().sys_mask().write(|w| w.mtxfrs(0b1))?;
        Ok(())
    }
}


/// Represents an RX operation that might not have finished
pub struct RxFuture<'r, SPI, CS>(&'r mut DW1000<SPI, CS, Ready>);

impl<'r, SPI, CS> RxFuture<'r, SPI, CS>
    where
        SPI: spi::Transfer<u8> + spi::Write<u8>,
        CS:  OutputPin,
{
    /// Wait for data to be available
    pub fn wait<'b>(&mut self, buffer: &'b mut [u8])
        -> nb::Result<Message<'b>, Error<SPI>>
    {
        // ATTENTION:
        // If you're changing anything about which SYS_STATUS flags are being
        // checked in this method, also make sure to update `enable_interrupts`.
        let sys_status = self.0.ll()
            .sys_status()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;

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

        // Frame is ready. Continue.

        // Wait until LDE processing is done. Before this is finished, the RX
        // time stamp is not available.
        if sys_status.ldedone() == 0b0 {
            return Err(nb::Error::WouldBlock);
        }
        let rx_time = self.0.ll()
            .rx_time()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?
            .rx_stamp();
        let rx_time = Instant(rx_time);

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
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;

        // Read received frame
        let rx_finfo = self.0.ll()
            .rx_finfo()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;
        let rx_buffer = self.0.ll()
            .rx_buffer()
            .read()
            .map_err(|error| nb::Error::Other(Error::Spi(error)))?;

        let len = rx_finfo.rxflen() as usize;

        if buffer.len() < len {
            return Err(nb::Error::Other(
                Error::BufferTooSmall { required_len: len }
            ))
        }

        buffer[..len].copy_from_slice(&rx_buffer.data()[..len]);

        let frame = mac::Frame::decode(&buffer[..len])
            .map_err(|error| nb::Error::Other(Error::Frame(error)))?;

        Ok(Message {
            rx_time,
            frame,
        })
    }

    /// Enables interrupts for the events that `wait` checks
    ///
    /// Overwrites any interrupt flags that were previously set.
    pub fn enable_interrupts(&mut self)
        -> Result<(), Error<SPI>>
    {
        self.0.ll()
            .sys_mask()
            .write(|w|
                w
                    .mrxdfr(0b1)
                    .mrxfce(0b1)
                    .mrxphe(0b1)
                    .mrxrfsl(0b1)
                    .mrxrfto(0b1)
                    .mrxovrr(0b1)
                    .mrxpto(0b1)
                    .mrxsfdto(0b1)
                    .mldedone(0b1)
            )?;

        Ok(())
    }
}


/// An error that can occur when sending or receiving data
pub enum Error<SPI>
    where SPI: spi::Transfer<u8> + spi::Write<u8>
{
    /// Error occured while using SPI bus
    Spi(ll::Error<SPI>),

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
    Frame(mac::DecodeError),

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

    /// An error occured while serializing or deserializing data
    Ssmarshal(ssmarshal::Error),
}

impl<SPI> From<ll::Error<SPI>> for Error<SPI>
    where SPI: spi::Transfer<u8> + spi::Write<u8>
{
    fn from(error: ll::Error<SPI>) -> Self {
        Error::Spi(error)
    }
}

impl<SPI> From<ssmarshal::Error> for Error<SPI>
    where SPI: spi::Transfer<u8> + spi::Write<u8>
{
    fn from(error: ssmarshal::Error) -> Self {
        Error::Ssmarshal(error)
    }
}

// We can't derive this implementation, as `Debug` is only implemented
// conditionally for `ll::Debug`.
impl<SPI> fmt::Debug for Error<SPI>
    where
        SPI: spi::Transfer<u8> + spi::Write<u8>,
        <SPI as spi::Transfer<u8>>::Error: fmt::Debug,
        <SPI as spi::Write<u8>>::Error: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Spi(error) =>
                write!(f, "Spi({:?})", error),
            Error::Fcs =>
                write!(f, "Fcs"),
            Error::Phy =>
                write!(f, "Phy"),
            Error::BufferTooSmall { required_len } =>
                write!(
                    f,
                    "BufferTooSmall {{ required_len: {:?} }}",
                    required_len,
                ),
            Error::ReedSolomon =>
                write!(f, "ReedSolomon"),
            Error::FrameWaitTimeout =>
                write!(f, "FrameWaitTimeout"),
            Error::Overrun =>
                write!(f, "Overrun"),
            Error::PreambleDetectionTimeout =>
                write!(f, "PreambleDetectionTimeout"),
            Error::SfdTimeout =>
                write!(f, "SfdTimeout"),
            Error::Frame(error) =>
                write!(f, "Frame({:?})", error),
            Error::DelayedSendTooLate =>
                write!(f, "DelayedSendTooLate"),
            Error::DelayedSendPowerUpWarning =>
                write!(f, "DelayedSendPowerUpWarning"),
            Error::Ssmarshal(error) =>
                write!(f, "Ssmarshal({:?})", error),
        }
    }
}


/// Indicates that the `DW1000` instance is not initialized yet
pub struct Uninitialized;

/// Indicates that the `DW1000` instance is ready to be used
pub struct Ready;


/// An incoming message
#[derive(Debug)]
pub struct Message<'l> {
    /// The time the message was received
    ///
    /// This time is based on the local system time, as defined in the SYS_TIME
    /// register.
    pub rx_time: Instant,

    /// The MAC frame
    pub frame: mac::Frame<'l>,
}


/// An instant, in DW1000 system time
///
/// DW1000 timestamps are 40-bit numbers. Creating an `Instant` with a value
/// larger than 2^40 - 1 can lead to undefined behavior.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Instant(pub u64);

/// A duration between two DW1000 system time instants
///
/// DW1000 timestamps are 40-bit numbers. Creating a `Duration` with a value
/// larger than 2^40 - 1 can lead to undefined behavior.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[repr(C)]
pub struct Duration(pub u64);

impl Add<Duration> for Instant {
    type Output = Instant;

    fn add(self, rhs: Duration) -> Self::Output {
        // Both `Instant` and `Duration` contain 40-bit numbers, so this
        // addition should never overflow.
        Instant((self.0 + rhs.0) % (TIME_MAX + 1))
    }
}
