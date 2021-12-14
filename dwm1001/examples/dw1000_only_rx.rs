//! Waits to receive data and signals status via LEDs

#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

use dwm1001::{
    block_timeout,
    dw1000::{
        ranging::{self, Message as _},
        RxConfig,
    },
    nrf52832_hal::{Delay, Timer},
    prelude::*,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("hello rx!");

    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();
    let mut delay = Delay::new(dwm1001.SYST);

    dwm1001.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = dwm1001
        .DW1000
        .init(&mut delay)
        .expect("Failed to initialize DW1000");

    // Configure timer
    let mut timer = Timer::new(dwm1001.TIMER0);

    loop {
        let mut receiving = dw1000
            .receive(RxConfig {
                frame_filtering: false,
                ..RxConfig::default()
            })
            .expect("Failed to start receiver");

        let mut buffer = [0; 1024];

        // Set timer for timeout
        timer.start(5_000_000u32);

        let result = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer));

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let message = match result {
            Ok(message) => message,
            Err(error) => match error {
                embedded_timeout_macros::TimeoutError::Timeout => {
                    defmt::debug!("Timeout");
                    continue;
                }
                embedded_timeout_macros::TimeoutError::Other(o) => {
                    defmt::debug!("Other error: {:?}", defmt::Debug2Format(&o));
                    continue;
                }
            },
        };

        defmt::info!("message successfully received");

        if message.frame.payload.starts_with(ranging::Ping::PRELUDE.0) {
            dwm1001.leds.D10.enable();
            delay.delay_ms(10u32);
            dwm1001.leds.D10.disable();
            continue;
        }
        if message
            .frame
            .payload
            .starts_with(ranging::Request::PRELUDE.0)
        {
            dwm1001.leds.D11.enable();
            delay.delay_ms(10u32);
            dwm1001.leds.D11.disable();
            continue;
        }
        if message
            .frame
            .payload
            .starts_with(ranging::Response::PRELUDE.0)
        {
            dwm1001.leds.D12.enable();
            delay.delay_ms(10u32);
            dwm1001.leds.D12.disable();
            continue;
        }

        dwm1001.leds.D9.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D9.disable();

        defmt::debug!("received frame {:#?}", defmt::Debug2Format(&message.frame));
    }
}
