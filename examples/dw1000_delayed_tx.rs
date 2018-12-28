//! Continually sends data using delayed transmission
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use nb::block;
use panic_semihosting;

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
        let tx_time = dw1000.time_from_delay(10_000_000) // ~10 ms
            .expect("Failed to compute transmission time");

        let mut tx = dw1000
            .send(b"ping", mac::Address::broadcast(), Some(tx_time))
            .expect("Failed to start receiver");

        print!("Sending... ");

        block!(tx.wait())
            .expect("Failed to send data");

        print!("done\n");
    }
}
