use crate::{Error, Ready, Sleeping, DW1000};
use embedded_hal::delay::DelayNs;
use embedded_hal::spi::SpiDevice;

impl<SPI> DW1000<SPI, Sleeping>
where
    SPI: SpiDevice,
{
    /// Wakes the radio up.
    pub fn wake_up(mut self, delay: &mut impl DelayNs) -> Result<DW1000<SPI, Ready>, Error<SPI>> {
        // Wake up using the spi
        self.ll.wake_up(850 * 2)?;

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
        let delay = self.state.tx_antenna_delay;
        self.ll.tx_antd().write(|w| w.value(delay.value() as u16))?;

        // All other values should be restored, so return the ready radio.
        Ok(DW1000 {
            ll: self.ll,
            seq: self.seq,
            state: Ready,
        })
    }
}
