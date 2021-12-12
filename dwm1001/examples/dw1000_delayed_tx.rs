//! Continually sends data using delayed transmission

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use dwm1001::{
    dw1000::{hl::SendTime, mac, time::Duration, TxConfig},
    nrf52832_hal::Delay,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    let dwm1001 = dwm1001::DWM1001::take().unwrap();
    let mut delay = Delay::new(dwm1001.SYST);
    let mut dw1000 = dwm1001.DW1000.init(&mut delay).unwrap();

    loop {
        let sys_time = dw1000.sys_time().expect("Failed to read system time");
        let tx_time = sys_time + Duration::from_nanos(10_000_000);

        let mut sending = dw1000
            .send(
                b"ping",
                mac::Address::broadcast(&mac::AddressMode::Short),
                SendTime::Delayed(tx_time),
                TxConfig::default(),
            )
            .expect("Failed to start receiver");

        defmt::info!("Sending... ");

        nb::block!(sending.wait_transmit()).expect("Failed to send data");

        dw1000 = sending.finish_sending().expect("Failed to finish sending");

        defmt::info!("done\n");
    }
}
