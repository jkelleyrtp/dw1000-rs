use crate::{Error, DW1000};
use embedded_hal::{blocking::spi, digital::v2::OutputPin};

use super::DW1000Status;

impl<SPI, CS> DW1000<SPI, CS>
where
    SPI: spi::Transfer<u8> + spi::Write<u8>,
    CS: OutputPin,
{
    /// Wakes the radio up.
    pub fn wake_up<DELAY: embedded_hal::blocking::delay::DelayUs<u16>>(
        &mut self,
        delay: &mut DELAY,
    ) -> Result<(), Error<SPI, CS>> {
        if let DW1000Status::Sleeping {
            tx_antenna_delay, ..
        } = self.state
        {
            // Wake up using the spi
            self.ll.assert_cs_low().map_err(|e| Error::Spi(e))?;
            delay.delay_us(850 * 2);
            self.ll.assert_cs_high().map_err(|e| Error::Spi(e))?;

            // Now we must wait 4 ms so all the clocks start running.
            delay.delay_us(4000 * 2);

            // Let's check that we're actually awake now
            if self.ll.dev_id().read()?.ridtag() != 0xDECA {
                // Oh dear... We have not woken up!
                return Err(Error::StillAsleep);
            }

            // Reset the wakeupstatus
            self.ll.sys_status().write(|w| w.slp2init(1).cplock(1))?;

            // Restore the tx antenna delay
            let delay = tx_antenna_delay;
            self.ll.tx_antd().write(|w| w.value(delay.value() as u16))?;

            // All other values should be restored, so return the ready radio.
            self.state = DW1000Status::Ready;
        }

        // if we're not sleeping, then we're already awake
        Ok(())
    }
}
