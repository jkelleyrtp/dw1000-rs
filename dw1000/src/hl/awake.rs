use crate::{
    ll, mac,
    time::{Duration, Instant},
    Error, DW1000,
};
use embedded_hal::{blocking::spi, digital::v2::OutputPin};

use super::Awake;

impl<SPI, CS, State> DW1000<SPI, CS, State>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
    State: Awake,
{
    /// Returns the TX antenna delay
    pub fn get_tx_antenna_delay(&mut self) -> Result<Duration, Error<SPI, CS>> {
        let tx_antenna_delay = self.ll.tx_antd().read()?.value();

        // Since `tx_antenna_delay` is `u16`, the following will never panic.
        let tx_antenna_delay = Duration::new(tx_antenna_delay.into()).unwrap();

        Ok(tx_antenna_delay)
    }

    /// Returns the RX antenna delay
    pub fn get_rx_antenna_delay(&mut self) -> Result<Duration, Error<SPI, CS>> {
        let rx_antenna_delay = self.ll.lde_rxantd().read()?.value();

        // Since `rx_antenna_delay` is `u16`, the following will never panic.
        let rx_antenna_delay = Duration::new(rx_antenna_delay.into()).unwrap();

        Ok(rx_antenna_delay)
    }

    /// Returns the network id and address used for sending and receiving
    pub fn get_address(&mut self) -> Result<mac::Address, Error<SPI, CS>> {
        let panadr = self.ll.panadr().read()?;

        Ok(mac::Address::Short(
            mac::PanId(panadr.pan_id()),
            mac::ShortAddress(panadr.short_addr()),
        ))
    }

    /// Returns the current system time
    pub fn sys_time(&mut self) -> Result<Instant, Error<SPI, CS>> {
        let sys_time = self.ll.sys_time().read()?.value();

        // Since hardware timestamps fit within 40 bits, the following should
        // never panic.
        Ok(Instant::new(sys_time).unwrap())
    }

    /// Provides direct access to the register-level API
    ///
    /// Be aware that by using the register-level API, you can invalidate
    /// various assumptions that the high-level API makes about the operation of
    /// the DW1000. Don't use the register-level and high-level APIs in tandem,
    /// unless you know what you're doing.
    pub fn ll(&mut self) -> &mut ll::DW1000<SPI, CS> {
        &mut self.ll
    }

    /// Force the DW1000 into IDLE mode
    ///
    /// Any ongoing RX/TX operations will be aborted.
    pub(super) fn force_idle(&mut self) -> Result<(), Error<SPI, CS>> {
        self.ll.sys_ctrl().write(|w| w.trxoff(0b1))?;
        while self.ll.sys_ctrl().read()?.trxoff() == 0b1 {}

        Ok(())
    }

    /// Checks whether the ldo tune is calibrated.
    ///
    /// The bool in the tuple is the answer and the int is the raw ldotune_low value.
    pub(super) fn is_ldo_tune_calibrated(&mut self) -> Result<(bool, u32), Error<SPI, CS>> {
        self.ll.otp_addr().write(|w| w.value(0x004))?;
        self.ll
            .otp_ctrl()
            .modify(|_, w| w.otprden(0b1).otpread(0b1))?;
        while self.ll.otp_ctrl().read()?.otpread() == 0b1 {}
        let ldotune_low = self.ll.otp_rdat().read()?.value();
        Ok((ldotune_low != 0, ldotune_low))
    }
}
