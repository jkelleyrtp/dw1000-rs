//! Range measurement mobile tag. To be used in tandem with `dw1000_ranging_basestation`
//!
//! This is a tag acting as a mobile tag, initiating pings to base stations. This tag loops every second polling
//! the base statinons in the area. A basestation will respond with a ranging request which the mobile tag will
//! respond with with a ranging response. The base station will then calculate the distance between the two devices
//! and log it to the console.
//!
//! This tag does not produce any useful console output.

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use dwm1001::{
    block_timeout,
    dw1000::{
        mac,
        ranging::{self, Message as _RangingMessage},
        RxConfig,
    },
    nrf52832_hal::{
        gpio::{p0::P0_17, Output, PushPull},
        pac::SPIM2,
        rng::Rng,
        Delay, Spim, Timer,
    },
    prelude::*,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("Launching basestation");

    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();

    let mut delay = Delay::new(dwm1001.SYST);
    let mut rng = Rng::new(dwm1001.RNG);

    dwm1001.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = dwm1001
        .DW1000
        .init(&mut delay)
        .expect("Failed to initialize DW1000");

    dw1000
        .enable_tx_interrupts()
        .expect("Failed to enable TX interrupts");

    dw1000
        .enable_rx_interrupts()
        .expect("Failed to enable RX interrupts");

    // These are the hardcoded calibration values from the dwm1001-examples
    // repository[1]. Ideally, the calibration values would be determined using
    // the proper calibration procedure, but hopefully those are good enough for
    // now.
    //
    // [1] https://github.com/Decawave/dwm1001-examples
    dw1000
        .set_antenna_delay(16456, 16300)
        .expect("Failed to set antenna delay");

    // Set network address
    dw1000
        .set_address(
            mac::PanId(0x0d57),                  // hardcoded network id
            mac::ShortAddress(rng.random_u16()), // random device address
        )
        .expect("Failed to set address");

    let mut timer = Timer::new(dwm1001.TIMER0);

    let mut buffer = [0; 1024];

    loop {
        /*
        Strategy for mobile tag ranging:
        - 1. Send a ping
        - 2. Wait for a ranging request
        - 3. Send a ranging response
        - 4. Delay to throttle the rate of pings
        */

        defmt::info!("Sending ping");

        dwm1001.leds.D10.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D10.disable();

        /*
        1. Send a ping
        */
        let mut sending = ranging::Ping::new(&mut dw1000)
            .expect("Failed to initiate ping")
            .send(dw1000)
            .expect("Failed to initiate ping transmission");

        nb::block!(sending.wait_transmit()).expect("Failed to send data");
        dw1000 = sending.finish_sending().expect("Failed to finish sending");

        defmt::info!("Ping sent, waiting for base station response");

        /*
        2. Wait for the anchor to respond with a ranging request.
        */
        let mut receiving = dw1000
            .receive(RxConfig::default())
            .expect("Failed to receive message");

        timer.start(5_000_000u32);
        let result = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer));

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let message = match result {
            Ok(message) => message,
            Err(error) => {
                use embedded_timeout_macros::TimeoutError;
                match error {
                    TimeoutError::Timeout => {
                        defmt::info!("Waiting for base station timed out. Trying again.")
                    }
                    TimeoutError::Other(o) => {
                        defmt::error!("Other error: {:?}", defmt::Debug2Format(&o));
                    }
                }
                continue;
            }
        };

        /*
        3. Decode the ranging request and respond with a ranging response
        */
        let request =
            match ranging::Request::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message) {
                Ok(Some(request)) => request,
                Ok(None) | Err(_) => {
                    defmt::info!("Ignoring message that is not a request\n");
                    continue;
                }
            };

        defmt::info!("Ranging request received. Preparing to send ranging response.");

        dwm1001.leds.D12.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D12.disable();

        // Wait for a moment, to give the tag a chance to start listening for
        // the reply.
        delay.delay_ms(10u32);

        // Send ranging response
        let mut sending = ranging::Response::new(&mut dw1000, &request)
            .expect("Failed to initiate response")
            .send(dw1000)
            .expect("Failed to initiate response transmission");

        nb::block!(sending.wait_transmit()).expect("Failed to send data");
        dw1000 = sending.finish_sending().expect("Failed to finish sending");

        defmt::info!("Ranging response sent");

        dwm1001.leds.D9.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D9.disable();

        /*
        - Throttle us to roughly 4 Hz
        */
        delay.delay_ms(250u32);
    }
}
