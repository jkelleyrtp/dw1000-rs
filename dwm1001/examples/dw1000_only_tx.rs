//! Continually sends data and signals its status via LEDs

#![no_main]
#![no_std]


extern crate panic_semihosting;


use cortex_m_rt::entry;
use nb::block;

use dwm1001::{
    debug,
    dw1000::{
        hl::SendTime,
        mac,
        TxConfig,
    },
    nrf52832_hal::Delay,
    DWM1001,
    print,
};


#[entry]
fn main() -> ! {
    debug::init();

    let     dwm1001 = DWM1001::take().unwrap();
    let mut delay   = Delay::new(dwm1001.SYST);
    let mut dw1000  = dwm1001.DW1000.init(&mut delay).unwrap();

    loop {
        let mut sending = dw1000
            .send(
                b"ping",
                mac::Address::broadcast(&mac::AddressMode::Short),
                SendTime::Now,
                TxConfig::default(),
            )
            .expect("Failed to start receiver");

        block!(sending.wait())
            .expect("Failed to send data");

        dw1000 = sending.finish_sending()
            .expect("Failed to finish sending");

        print!(".");
    }
}
