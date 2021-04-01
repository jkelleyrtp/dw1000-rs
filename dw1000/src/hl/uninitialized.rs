use crate::{ll, Error, Ready, Uninitialized, DW1000};
use core::num::Wrapping;
use embedded_hal::{blocking::spi, digital::v2::OutputPin};

impl<SPI, CS> DW1000<SPI, CS, Uninitialized>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    /// Create a new instance of `DW1000`
    ///
    /// Requires the SPI peripheral and the chip select pin that are connected
    /// to the DW1000.
    pub fn new(spi: SPI, chip_select: CS) -> Self {
        DW1000 {
            ll: ll::DW1000::new(spi, chip_select),
            seq: Wrapping(0),
            state: Uninitialized,
        }
    }

    /// Set the chip select delay.
    ///
    /// This is the amount of times the cs pin will be set low before any data is transfered.
    /// This way, the chip can be used on fast mcu's just fine.
    pub fn with_cs_delay(mut self, delay: u8) -> Self {
        self.ll.set_chip_select_delay(delay);
        self
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
    pub fn init(mut self) -> Result<DW1000<SPI, CS, Ready>, Error<SPI, CS>> {
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

        // Set LDELOAD. See user manual, section 2.5.5.10.
        self.ll
            .pmsc_ctrl0()
            .modify(|r, w| w.raw_value(r.raw_value() | 0x0301))?;
        self.ll.otp_ctrl().write(|w| w.ldeload(0b1))?;
        while self.ll.otp_ctrl().read()?.ldeload() == 0b1 {}
        self.ll
            .pmsc_ctrl0()
            .modify(|r, w| w.raw_value(r.raw_value() & !0x0101))?;

        // Set LDOTUNE. See user manual, section 2.5.5.11.
        let (calibrated, ldotune_low) = self.is_ldo_tune_calibrated()?;
        if calibrated {
            self.ll.otp_addr().write(|w| w.value(0x005))?;
            self.ll
                .otp_ctrl()
                .modify(|_, w| w.otprden(0b1).otpread(0b1))?;
            while self.ll.otp_ctrl().read()?.otpread() == 0b1 {}
            let ldotune_high = self.ll.otp_rdat().read()?.value();

            let ldotune = ldotune_low as u64 | (ldotune_high as u64) << 32;
            self.ll.ldotune().write(|w| w.value(ldotune))?;
        }

        Ok(DW1000 {
            ll: self.ll,
            seq: self.seq,
            state: Ready,
        })
    }
}
