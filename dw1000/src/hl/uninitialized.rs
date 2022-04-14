use crate::{ll, Error, RxConfig, TxConfig, DW1000};
use core::num::Wrapping;
use embedded_hal::{
    blocking::{delay::DelayMs, spi},
    digital::v2::OutputPin,
};

/// A Radio before initialization
///
/// Calling "initialize" will give us an actual DWM radio that can have methods called on it
pub struct UninitializedDW1000<SPI, CS> {
    ll: ll::DW1000<SPI, CS>,
    seq: Wrapping<u8>,
}

impl<SPI, CS> UninitializedDW1000<SPI, CS>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    /// Create a new instance of `DW1000`
    ///
    /// Requires the SPI peripheral and the chip select pin that are connected
    /// to the DW1000.
    pub fn new(spi: SPI, chip_select: CS) -> Self {
        UninitializedDW1000 {
            ll: ll::DW1000::new(spi, chip_select),
            seq: Wrapping(0),
        }
    }

    /// Get the low-level interface to the uninitialized DWM1000
    pub fn ll(&mut self) -> &mut ll::DW1000<SPI, CS> {
        &mut self.ll
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
    pub fn init<D: DelayMs<u8>>(
        mut self,
        delay: &mut D,
    ) -> Result<DW1000<SPI, CS>, Error<SPI, CS>> {
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
        self.ll
            .rf_txctrl()
            .modify(|_, w| w.txmtune(0b1111).txmq(0b111))?;

        // Set TC_PGDELAY. See user manual, section 2.5.5.8.
        self.ll.tc_pgdelay().write(|w| w.value(0xC0))?;

        // Set FS_PLLTUNE. See user manual, section 2.5.5.9.
        self.ll.fs_plltune().write(|w| w.value(0xBE))?;

        // Set LDOTUNE. See user manual, section 2.5.5.11.
        let ldotune_low = self.read_otp(0x004)?;
        if ldotune_low != 0 {
            let ldotune_high = self.read_otp(0x005)?;
            let ldotune = ldotune_low as u64 | (ldotune_high as u64) << 32;
            self.ll.ldotune().write(|w| w.value(ldotune))?;
        }

        // Set LDELOAD. See user manual, section 2.5.5.10.
        self.ll
            .pmsc_ctrl0()
            .modify(|r, w| w.raw_value(r.raw_value() | 0x0301))?;
        self.ll.otp_ctrl().write(|w| w.ldeload(0b1))?;
        delay.delay_ms(5);
        self.ll
            .pmsc_ctrl0()
            .modify(|r, w| w.raw_value(r.raw_value() & !0x0101))?;

        Ok(DW1000 {
            ll: self.ll,
            seq: self.seq,
            state: super::DW1000Status::Ready,
            rx_cfg: RxConfig::default(),
            tx_cfg: TxConfig::default(),
        })
    }

    pub(crate) fn read_otp(&mut self, address: u16) -> Result<u32, Error<SPI, CS>> {
        // Set address
        self.ll.otp_addr().write(|w| w.value(address))?;
        // Switch into read mode
        self.ll.otp_ctrl().write(|w| w.otprden(0b1).otpread(0b1))?;
        self.ll.otp_ctrl().write(|w| w.otprden(0b1))?;
        // Read back value
        let value = self.ll.otp_rdat().read()?.value();
        // End read mode
        self.ll.otp_ctrl().write(|w| w)?;
        Ok(value)
    }
}
