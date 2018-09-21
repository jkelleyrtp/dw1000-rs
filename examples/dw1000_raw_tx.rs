//! Continually sends data and signals its status via LEDs


#![no_main]
#![no_std]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate dwm1001;
#[macro_use] extern crate nb;

extern crate cortex_m_semihosting;
extern crate panic_semihosting;


use dwm1001::{
    debug,
    DWM1001,
};


#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    loop {
        let mut tx = dwm1001.DW1000
            .send_raw(b"ping")
            .expect("Failed to start receiver");

        block!(tx.wait())
            .expect("Failed to send data");

        print!(".");
    }
}
