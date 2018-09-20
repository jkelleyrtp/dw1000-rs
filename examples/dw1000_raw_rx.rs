//! Waits to receive data and signals status via LEDs
//!
//! Please note that, for unknown reasons, this example only receives data once,
//! then needs to be re-flashed to receive data again. It's a start.


#![no_main]
#![no_std]


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
        let mut receiver = dwm1001.DW1000
            .start_receiver()
            .expect("Failed to start receiver");

        let mut buffer = [0; 1024];

        // Set timer for timeout
        timer.start(5_000_000);

        // Wait until frame has been received
        let len = loop {
            match receiver.receive(&mut buffer) {
                Ok(len) =>
                    break len,
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

        let data = &buffer[..len];

        print!("Received data: {:?}\n", data);

        let expected_data = b"ping";

        // Received data should have length of expected data, plus 2-byte CRC
        // checksum.
        if data.len() != expected_data.len() + 2 {
            print!("Unexpected length: {}\n", data.len());
            continue;
        }

        if data[0 .. data.len() - 2] != expected_data[..] {
            print!("Unexpected data");
            continue;
        }

        // Signal that data was received
        for _ in 0..20 {
            dwm1001.leds.D10.enable();
            delay.delay_ms(30u32);
            dwm1001.leds.D10.disable();
            delay.delay_ms(30u32);
        }
    }
}
