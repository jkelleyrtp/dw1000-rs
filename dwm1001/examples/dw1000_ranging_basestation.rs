//! Range measurement basestation. To be used in tandem with `dw1000_ranging_mobile_tag`
//!
//! This is a tag acting as a base station, collecting distances to mobile tags.
//!
//! The anchor/tag example does the distance calculation *at the tag* which is less useful for applications where
//! the tags are very "dumb".
//!
//! Instead, the basestation intiates the ranging request and records the distance over defmt.

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
    defmt::debug!("Launching basestation.");

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

    let mut buffer1 = [0; 1024];
    let mut buffer2 = [0; 1024];

    loop {
        /*
        Strategy for basestation:
        - 1. Wait for ping
        - 2. Initiate ranging request
        - 3. Wait for response
        - 4. Calculate and log distance to tag
        */

        defmt::debug!("Waiting for mobile tag ping.");

        /*
        1. Wait for ping
        */
        let mut receiving = dw1000
            .receive(RxConfig::default())
            .expect("Failed to receive message");

        let message = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer1));

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let message = match message {
            Ok(message) => message,
            Err(_) => {
                defmt::error!("Timeout error occured");
                continue;
            }
        };

        let ping = match ranging::Ping::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message) {
            Ok(Some(ping)) => ping,
            Ok(None) => {
                defmt::error!("Failed to decode ping");
                continue;
            }
            Err(e) => {
                defmt::error!("Ping decode error: {:?}", defmt::Debug2Format(&e));
                continue;
            }
        };

        defmt::debug!(
            "Received ping from {:?}.\nResponding with ranging request.",
            ping.source
        );
        dwm1001.leds.D10.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D10.disable();

        // Wait for a moment, to give the anchor a chance to start listening
        // for the reply.
        delay.delay_ms(10u32);

        /*
        2. Initiate ranging request
        */
        let mut sending = ranging::Request::new(&mut dw1000, &ping)
            .expect("Failed to initiate request")
            .send(dw1000)
            .expect("Failed to initiate request transmission");

        nb::block!(sending.wait_transmit()).expect("Failed to send data");
        dw1000 = sending.finish_sending().expect("Failed to finish sending");

        defmt::debug!("Request sent Transmission sent. Waiting for response.");

        /*
        3. Wait for response
        */
        let mut receiving = dw1000
            .receive(RxConfig::default())
            .expect("Failed to receive message");

        // Set timer for timeout
        timer.start(5_000_000u32);
        let result = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer2));

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let message = match result {
            Ok(message) => message,
            Err(error) => match error {
                embedded_timeout_macros::TimeoutError::Timeout => {
                    defmt::debug!("Waiting for mobile tag respond timed out.");
                    continue;
                }
                embedded_timeout_macros::TimeoutError::Other(other) => {
                    defmt::error!("Other timeout error: {:?}", defmt::Debug2Format(&other));
                    continue;
                }
            },
        };

        let response =
            match ranging::Response::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message) {
                Ok(Some(response)) => response,
                Ok(None) => {
                    defmt::error!(
                        "Failed to decode ranging response. Frame is {:?}",
                        defmt::Debug2Format(&message.frame)
                    );
                    continue;
                }
                Err(e) => {
                    defmt::error!(
                        "Ranging response decode error: {:?}",
                        defmt::Debug2Format(&e)
                    );
                    continue;
                }
            };

        /*
        4. Calculate distance
        */
        dwm1001.leds.D11.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D11.disable();

        // If this is not a PAN ID and short address, it doesn't
        // come from a compatible node. Ignore it.
        let (pan_id, addr) = match response.source {
            Some(mac::Address::Short(pan_id, addr)) => (pan_id, addr),
            _ => continue,
        };

        // Ranging response received. Compute distance.
        match ranging::compute_distance_mm(&response) {
            Ok(distance_mm) => {
                dwm1001.leds.D9.enable();
                delay.delay_ms(10u32);
                dwm1001.leds.D9.disable();

                defmt::info!("{:04x}:{:04x} - {} mm\n", pan_id.0, addr.0, distance_mm,);
            }
            Err(e) => {
                defmt::error!("Ranging response error: {:?}", defmt::Debug2Format(&e));
            }
        }
    }
}
