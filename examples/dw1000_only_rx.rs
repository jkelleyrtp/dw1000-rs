//! Waits to receive data and signals status via LEDs


#![no_main]
#![no_std]

#![feature(nll)]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate dwm1001;

extern crate cortex_m_semihosting;
extern crate nb;
extern crate panic_semihosting;


use dwm1001::{
    prelude::*,
    debug,
    nrf52832_hal::Delay,
    DWM1001,
};


#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    let     clocks = dwm1001.CLOCK.constrain().freeze();
    let mut delay  = Delay::new(dwm1001.SYST, clocks);

    dwm1001.DW_RST.reset_dw1000(&mut delay);

    // Configure timer
    let mut timer = dwm1001.TIMER0.constrain();

    'outer: loop {
        let mut rx = dwm1001.DW1000
            .receive()
            .expect("Failed to start receiver");

        let mut buffer = [0; 1024];

        // Set timer for timeout
        timer.start(5_000_000);

        // Wait until frame has been received
        let frame = loop {
            match rx.wait(&mut buffer) {
                Ok(frame) =>
                    break frame,
                Err(nb::Error::WouldBlock) =>
                    (),
                Err(error) => {
                    print!("Error: {:?}\n", error);
                    continue 'outer;
                }
            }

            match timer.wait() {
                Ok(()) => {
                    print!("Timeout\n");
                    continue 'outer;
                }
                Err(nb::Error::WouldBlock) =>
                    (),
                Err(error) =>
                    panic!("Failed to wait for timer: {:?}", error),
            }
        };

        print!("Received frame: {:x?}\n", frame);

        // Signal that data was received
        for _ in 0..20 {
            dwm1001.leds.D10.enable();
            delay.delay_ms(30u32);
            dwm1001.leds.D10.disable();
            delay.delay_ms(30u32);
        }
    }
}
