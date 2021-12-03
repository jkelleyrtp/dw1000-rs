//! Range measurement anchor node
//!
//! This is an anchor node used for range measurement. Anchors have a known
//! location, and provide the support infrastructure requires by tag nodes to
//! determine their own distance from the available anchors.
//!
//! Currently, distance measurements have a highly inaccurate result. One reason
//! that could account for this is the lack of antenna delay calibration, but
//! it's possible that there are various hidden bugs that contribute to this.

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
    defmt::info!("Launching anchor");

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

    let mut task_timer = Timer::new(dwm1001.TIMER0);
    let mut timeout_timer = Timer::new(dwm1001.TIMER1);

    defmt::info!("Timer started");
    task_timer.start(1_000_000u32);

    let mut buf = [0; 128];

    let mut frame_id = 0;
    let mut ping_id = 0;

    loop {
        /*
        Strategy:
        - Sending a ranging ping
        - Waiting for a ranging request
        - Responding with a ranging response





        */

        // After receiving for a while, it's time to send out a ping
        if let Ok(()) = task_timer.wait() {
            defmt::info!("Sending ping {}", ping_id);
            ping_id += 1;
            task_timer.start(5_000_000u32);

            dwm1001.leds.D10.enable();
            delay.delay_ms(10u32);
            dwm1001.leds.D10.disable();

            let mut sending = ranging::Ping::new(&mut dw1000)
                .expect("Failed to initiate ping")
                .send(dw1000)
                .expect("Failed to initiate ping transmission");

            timeout_timer.start(100_000u32);
            nb::block!({
                dw_irq.wait_for_interrupts(&mut gpiote, &mut timeout_timer);
                sending.wait_transmit()
            })
            .expect("Failed to send ping");

            dw1000 = sending.finish_sending().expect("Failed to finish sending");
        }

        defmt::info!("Starting receive. Frame ID: {}", frame_id);
        frame_id += 1;

        let mut receiving = dw1000
            .receive(RxConfig::default())
            .expect("Failed to receive message");

        timeout_timer.start(500_000u32);

        let result = block_timeout!(&mut timeout_timer, {
            dw_irq.wait_for_interrupts(&mut gpiote, &mut timeout_timer);
            receiving.wait_receive(&mut buf)
        });

        dw1000 = receiving
            .finish_receiving()
            .expect("Failed to finish receiving");

        let message = match result {
            Ok(message) => message,
            _ => {
                defmt::info!("Msg not found");
                continue;
            }
        };

        defmt::info!("Response found");

        dwm1001.leds.D11.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D11.disable();

        let request = ranging::Request::decode::<Spim<SPIM2>, P0_17<Output<PushPull>>>(&message);

        let request = match request {
            Ok(Some(request)) => request,
            Ok(None) | Err(_) => {
                defmt::info!("Ignoring message that is not a request\n");
                continue;
            }
        };

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
        timeout_timer.start(100_000u32);
        nb::block!({
            dw_irq.wait_for_interrupts(&mut gpiote, &mut timeout_timer);
            sending.wait_transmit()
        })
        .expect("Failed to send ranging response");

        dw1000 = sending.finish_sending().expect("Failed to finish sending");

        dwm1001.leds.D9.enable();
        delay.delay_ms(10u32);
        dwm1001.leds.D9.disable();
    }
}
