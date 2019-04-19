//! Waits to receive data and signals status via LEDs

#![no_main]
#![no_std]


extern crate panic_semihosting;


use cortex_m_rt::entry;
use nb;

use dwm1001::{
    block_timeout,
    debug,
    DWM1001,
    nrf52832_hal::Delay,
    prelude::*,
    print,
};


#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    let     clocks = dwm1001.CLOCK.constrain();
    let mut delay  = Delay::new(dwm1001.SYST);

    dwm1001.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = dwm1001.DW1000.init()
        .expect("Failed to initialize DW1000");

    // Configure timer
    let mut timer = dwm1001.TIMER0.constrain();

    'outer: loop {
        let mut rx = dw1000
            .receive()
            .expect("Failed to start receiver");

        let mut buffer = [0; 1024];

        // Set timer for timeout
        timer.start(5_000_000u32);

        let message = match block_timeout!(&mut timer, rx.wait(&mut buffer)) {
            Ok(message) => {
                message
            }
            Err(error) => {
                print!("Error: {:?}\n", error);
                continue;
            }
        };

        print!("Received frame: {:x?}\n", message.frame);

        // Signal that data was received
        for _ in 0..20 {
            dwm1001.leds.D10.enable();
            delay.delay_ms(30u32);
            dwm1001.leds.D10.disable();
            delay.delay_ms(30u32);
        }
    }
}
