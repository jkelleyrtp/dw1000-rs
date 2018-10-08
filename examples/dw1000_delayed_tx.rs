//! Continually sends data using delayed transmission


#![no_main]
#![no_std]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate dwm1001;
#[macro_use] extern crate nb;

extern crate panic_semihosting;


use dwm1001::{
    debug,
    dw1000::mac,
    DWM1001,
};


#[entry]
fn main() -> ! {
    debug::init();

    let     dwm1001 = DWM1001::take().unwrap();
    let mut dw1000  = dwm1001.DW1000.init().unwrap();

    loop {
        let sys_time = dw1000.ll()
            .sys_time()
            .read()
            .expect("Failed to read system time")
            .value();

        let delay   = 10 * 64_000_000; // ~10 ms
        let tx_time = sys_time + delay;

        let mut tx = dw1000
            .send(b"ping", mac::Address::broadcast(), Some(tx_time))
            .expect("Failed to start receiver");

        print!("Sending... ");

        block!(tx.wait())
            .expect("Failed to send data");

        print!("done\n");
    }
}
