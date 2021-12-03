//! Range measurement basestation
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
    defmt::debug!("Launching basestation");

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

    let mut dw_irq = dwm1001.DW_IRQ;
    let mut gpiote = dwm1001.GPIOTE;

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
        - wait for ping
        - initiate ranging request
        - wait for response
        - calculate distance
        - log it
        */

        defmt::debug!("waiting for base mobile tag ping");

        let mut receiving = dw1000
            .receive(RxConfig::default())
            .expect("Failed to receive message");

        let message = block_timeout!(&mut timer, receiving.wait_receive(&mut buffer1));

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let message = match message {
            Ok(message) => message,
            Err(e) => {
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

        // Received ping from an anchor. Reply with a ranging
        defmt::debug!("Received ping. Responding with ranging request.");

        dwm1001.leds.D10.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D10.disable();

        // Wait for a moment, to give the anchor a chance to start listening
        // for the reply.
        delay.delay_ms(10u32);

        let mut sending = ranging::Request::new(&mut dw1000, &ping)
            .expect("Failed to initiate request")
            .send(dw1000)
            .expect("Failed to initiate request transmission");

        nb::block!(sending.wait_transmit()).expect("Failed to send data");
        dw1000 = sending.finish_sending().expect("Failed to finish sending");

        defmt::debug!("Request sent Transmission sent. Waiting for response.");

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
            Err(error) => {
                use embedded_timeout_macros::TimeoutError;
                match error {
                    TimeoutError::Timeout => {
                        defmt::debug!("Waiting for base station timed out. Trying again.")
                    }
                    TimeoutError::Other(o) => {
                        defmt::error!("Other error: {:?}", defmt::Debug2Format(&o));
                    }
                }
                continue;
            }
        };

        let response =
            match ranging::Response::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message) {
                Ok(Some(response)) => response,
                Ok(None) => {
                    // Frame {
                    //     header: Header {
                    //         frame_type: Data,
                    //         frame_pending: false,
                    //         ack_request: false,
                    //         pan_id_compress: false,
                    //         seq_no_suppress: false,
                    //         ie_present: false,
                    //         version: Ieee802154_2006,
                    //         seq: 129,
                    //         destination: Some(Short(PanId(65535), ShortAddress(65535))),
                    //         source: Some(Short(PanId(3415), ShortAddress(6325))),
                    //         auxiliary_security_header: None,
                    //     },
                    //     content: Data,
                    //     payload: [
                    //         82, 65, 78, 71, 73, 78, 71, 32, 80, 73, 78, 71, 172, 91, 228, 168, 99,
                    //         0, 0, 0,
                    //     ],
                    //     footer: [189, 146],
                    // };

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

        dwm1001.leds.D11.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D11.disable();

        // If this is not a PAN ID and short address, it doesn't
        // come from a compatible node. Ignore it.
        let (pan_id, addr) = match response.source {
            Some(mac::Address::Short(pan_id, addr)) => (pan_id, addr),
            _ => continue,
        };

        let ping_rt = response.payload.ping_reply_time.value();
        let ping_rtt = response.payload.ping_round_trip_time.value();
        let request_rt = response.payload.request_reply_time.value();
        let request_rtt = response
            .rx_time
            .duration_since(response.payload.request_tx_time)
            .value();

        defmt::info!(
            r#"
        Ping reply time: {:?}
        Ping round trip time: {:?}
        Request reply time: {:?}
        Request round trip time: {:?}
        "#,
            ping_rt,
            ping_rtt,
            request_rt,
            request_rtt
        );

        // Compute time of flight according to the formula given in the DW1000 user
        // manual, section 12.3.2.
        let rtt_product = ping_rtt.checked_mul(request_rtt).unwrap();
        // .ok_or(ComputeDistanceError::RoundTripTimesTooLarge)?;
        let rt_product = ping_rt.checked_mul(request_rt).unwrap();
        // .ok_or(ComputeDistanceError::ReplyTimesTooLarge)?;
        let rt_sum = ping_rt.checked_add(request_rt).unwrap();
        // .ok_or(ComputeDistanceError::SumTooLarge)?;
        let rtt_sum = ping_rtt.checked_add(request_rtt).unwrap();
        // .ok_or(ComputeDistanceError::SumTooLarge)?;
        let sum = rt_sum.checked_add(rtt_sum).unwrap();
        // .ok_or(ComputeDistanceError::SumTooLarge)?;

        defmt::info!(
            r#"
            rtt_product: {:?} 
            rt_product: {:?} 
            rt_sum: {:?} 
            rtt_sum: {:?} 
            sum: {:?}
            "#,
            rtt_product,
            rt_product,
            rt_sum,
            rtt_sum,
            sum
        );

        let time_diff = (rtt_product - rt_product);
        let time_of_flight = time_diff.checked_div(sum);
        if time_of_flight.is_none() {
            defmt::error!("Time of flight is too large");
            continue;
        }

        // Ranging response received. Compute distance.
        // match ranging::compute_distance_mm(&response) {
        //     Ok(distance_mm) => {
        //         dwm1001.leds.D9.enable();
        //         delay.delay_ms(10u32);
        //         dwm1001.leds.D9.disable();

        //         defmt::info!("{:04x}:{:04x} - {} mm\n", pan_id.0, addr.0, distance_mm,);
        //     }
        //     Err(e) => {
        //         defmt::error!("Ranging response error: {:?}", defmt::Debug2Format(&e));
        //     }
        // }
    }
}
