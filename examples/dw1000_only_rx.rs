//! Waits to receive data and signals status via LEDs

#![no_main]
#![no_std]


extern crate panic_semihosting;


use cortex_m_rt::entry;

use dwm1001::{
    block_timeout,
    debug,
    dw1000::RxConfig,
    DWM1001,
    nrf52832_hal::Delay,
    prelude::*,
    print,
};


#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();
    let mut delay  = Delay::new(dwm1001.SYST);

    dwm1001.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = dwm1001.DW1000.init()
        .expect("Failed to initialize DW1000");

    // Configure timer
    let mut timer = dwm1001.TIMER0.constrain();

    loop {
        let mut receiving = dw1000
            .receive(RxConfig {
                frame_filtering: false,
                .. RxConfig::default()
            })
            .expect("Failed to start receiver");

        let mut buffer = [0; 1024];

        // Set timer for timeout
        timer.start(5_000_000u32);

        let result = block_timeout!(&mut timer, receiving.wait(&mut buffer));

        dw1000 = receiving.finish_receiving()
            .expect("Failed to finish receiving");

        let message = match result {
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
