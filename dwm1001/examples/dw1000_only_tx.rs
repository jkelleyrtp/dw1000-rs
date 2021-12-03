//! Continually sends data and signals its status via LEDs

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use dwm1001::{
    dw1000::{hl::SendTime, mac, TxConfig},
    nrf52832_hal::Delay,
    DWM1001,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("hello tx!");

    let dwm1001 = dwm1001::DWM1001::take().unwrap();
    let mut delay = Delay::new(dwm1001.SYST);
    let mut dw1000 = dwm1001.DW1000.init(&mut delay).unwrap();

    let mut cast = 0;
    loop {
        cast += 1;
        let mut sending = dw1000
            .send(
                b"ping",
                mac::Address::broadcast(&mac::AddressMode::Short),
                SendTime::Now,
                TxConfig::default(),
            )
            .expect("Failed to start receiver");

        nb::block!(sending.wait_transmit()).expect("Failed to send data");

        dw1000 = sending.finish_sending().expect("Failed to finish sending");

        if cast % 10 == 0 {
            defmt::info!("{}", cast);
        }
    }
}
