//! Continually sends data and signals its status via LEDs

#![no_main]
#![no_std]


extern crate panic_semihosting;


use cortex_m_rt::entry;
use nb::block;

use dwm1001::{
    debug,
    dw1000::mac,
    DWM1001,
    print,
};


#[entry]
fn main() -> ! {
    debug::init();

    let     dwm1001 = DWM1001::take().unwrap();
    let mut dw1000  = dwm1001.DW1000.init().unwrap();

    loop {
        let mut tx = dw1000
            .send(
                b"ping",
                mac::Address::broadcast(&mac::AddressMode::Short),
                None,
            )
            .expect("Failed to start receiver");

        block!(tx.wait())
            .expect("Failed to send data");

        print!(".");
    }
}
