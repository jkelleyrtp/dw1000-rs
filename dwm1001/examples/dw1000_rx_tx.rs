//! Sends and receives data continuously
//!
//! This data sends and receives data continuously. To see it working in its
//! full glory, you need two DWM1001-DEV boards running it.
//!
//! As printing debug output via semihosting is really slow and would interefere
//! with the rest of the code, this example signals its status via LEDs. If
//! everything works well, you should see the blue LED blink from time to time
//! on both boards, signalling a successfully received message.

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use dw1000::hl::SendTime;
use heapless::FnvIndexSet;

use dwm1001::{
    block_timeout,
    dw1000::{mac, RxConfig, TxConfig},
    nrf52832_hal::{rng::Rng, Delay, Timer},
    prelude::*,
    repeat_timeout,
};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut known_nodes = FnvIndexSet::<_, 64>::new();

    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();

    let mut delay = Delay::new(dwm1001.SYST);
    let mut rng = Rng::new(dwm1001.RNG);

    dwm1001.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = dwm1001
        .DW1000
        .init(&mut delay)
        .expect("Failed to initialize DW1000");

    // Set network address
    dw1000
        .set_address(
            mac::PanId(0x0d57),                  // hardcoded network id
            mac::ShortAddress(rng.random_u16()), // random device address
        )
        .expect("Failed to set address");

    // Configure timer
    let mut task_timer = Timer::new(dwm1001.TIMER0);
    let mut timeout_timer = Timer::new(dwm1001.TIMER1);
    let mut output_timer = Timer::new(dwm1001.TIMER2);

    let receive_time = 500_000 + (rng.random_u32() % 500_000);

    output_timer.start(5_000_000u32);

    loop {
        let mut buffer = [0u8; 1024];
        let leds = &mut dwm1001.leds;

        task_timer.start(receive_time);
        repeat_timeout!(
            &mut task_timer,
            {
                let mut receiving = dw1000
                    .receive(RxConfig::default())
                    .expect("Failed to receive");

                timeout_timer.start(100_000u32);
                let result = block_timeout!(
                    &mut timeout_timer,
                    receiving.wait_receive(&mut buffer)
                );

                dw1000 = receiving.finish_receiving()
                    .expect("Failed to finish receiving");

                result
            },
            (message) {
                if message.frame.payload != b"ping" {
                    continue;
                }

                // Sucessfully received: Blink blue LED
                leds.D10.enable();
                delay.delay_ms(30u32);
                leds.D10.disable();

                // Unfortunately we can't just put the address into the map, due
                // to limitations in the `hash32` crate. We're not expecting any
                // messages from other kinds of nodes, so let's just assume this
                // is going to be a PAN ID and short address.
                let source = match message.frame.header.source {
                    Some(mac::Address::Short(pan_id, address)) =>
                        [pan_id.0, address.0],
                    _ =>
                        continue,
                };

                if let Err(_) = known_nodes.insert(source) {
                    defmt::info!("Too many nodes. Can't add another one.\n");
                }
            };
            (_error) {};
        );

        task_timer.start(50_000u32);
        repeat_timeout!(
            &mut task_timer,
            {
                let mut sending = dw1000
                    .send(
                        b"ping",
                        mac::Address::broadcast(&mac::AddressMode::Short),
                        SendTime::Now,
                        TxConfig::default()
                    )
                    .expect("Failed to broadcast ping");

                timeout_timer.start(10_000u32);
                let result = block_timeout!(&mut timeout_timer, sending.wait_transmit());

                dw1000 = sending.finish_sending()
                    .expect("Failed to finish sending");

                result
            },
            (_message) {};
            (_error) {};
        );

        if output_timer.wait().is_ok() {
            defmt::info!("\n-- Known nodes:\n");
            for node in &known_nodes {
                defmt::info!(
                    "PAN ID: 0x{:04x}, Short Address: 0x{:04x}\n",
                    node[0],
                    node[1],
                );
            }

            output_timer.start(5_000_000u32);
        }
    }
}
