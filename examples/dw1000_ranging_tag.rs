//! Range measurement tag node
//!
//! This is a tag node used for range measurement. Tags use anchor nodes to
//! measure their distance from those anchors.
//!
//! Currently, distance measurements have a highly inaccurate result. One reason
//! that could account for this is the lack of antenna delay calibration, but
//! it's possible that there are various hidden bugs that contribute to this.


#![no_main]
#![no_std]

#![feature(nll)]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate dwm1001;
#[macro_use] extern crate nb;

extern crate cortex_m_semihosting;
extern crate panic_semihosting;


use dwm1001::{
    prelude::*,
    debug,
    dw1000::{
        mac,
        ranging::{
            self,
            Message as _RangingMessage,
        },
        util::duration_between,
        Message,
    },
    nrf52832_hal::Delay,
    DWM1001,
};


#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    let     clocks = dwm1001.CLOCK.constrain().freeze();
    let mut delay  = Delay::new(dwm1001.SYST, clocks);
    let mut rng    = dwm1001.RNG.constrain();

    dwm1001.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = dwm1001.DW1000.init()
        .expect("Failed to initialize DW1000");

    // Set network address
    dw1000
        .set_address(
            mac::Address {
                pan_id:     0x0d57,           // hardcoded network id
                short_addr: rng.random_u16(), // random device address
            }
        )
        .expect("Failed to set address");

    let mut task_timer    = dwm1001.TIMER0.constrain();
    let mut timeout_timer = dwm1001.TIMER1.constrain();

    loop {
        let mut buf = [0; 128];

        // Listen for messages
        task_timer.start(100_000);
        repeat_timeout!(
            &mut task_timer,
            {
                let mut future = dw1000
                    .receive()
                    .expect("Failed to receive message");

                timeout_timer.start(100_000);
                block_timeout!(&mut timeout_timer, future.wait(&mut buf))
            },
            |message: Message| {
                let ping = ranging::Ping::decode(message.frame.payload)
                    .expect("Failed to decode ping");
                if let Some(ping) = ping {
                    // Received ping from an anchor. Reply with a ranging
                    // request.

                    let mut future = ranging::Request
                        ::initiate(
                            &mut dw1000,
                            ping.ping_tx_time,
                            message.rx_time,
                            message.frame.header.source,
                        )
                        .expect("Failed to initiate request")
                        .send(&mut dw1000)
                        .expect("Failed to initiate request transmission");

                    block!(future.wait())
                        .expect("Failed to send ranging request");

                    return;
                }

                let response = ranging::Response::decode(message.frame.payload)
                    .expect("Failed to decode response");
                if let Some(response) = response {
                    // Ranging response received. Compute distance.

                    let request_round_trip_time = duration_between(
                        response.request_tx_time,
                        message.rx_time,
                    );

                    let rtt_product =
                        response.ping_round_trip_time.0 *
                        request_round_trip_time.0;
                    let reply_time_product =
                        response.ping_reply_time.0 *
                        response.request_reply_time.0;
                    let complete_sum =
                        response.ping_round_trip_time.0 +
                        request_round_trip_time.0 +
                        response.ping_reply_time.0 +
                        response.request_reply_time.0;

                    let time_of_flight =
                        (rtt_product - reply_time_product) / complete_sum;

                    // Nominally, all time units are based on a 64 Ghz clock,
                    // meaning each time unit is 1/64 ns.

                    const SPEED_OF_LIGHT: u64 = 299_792_458; // m/s or nm/ns

                    let distance_nm_times_64 =
                        SPEED_OF_LIGHT.checked_mul(time_of_flight);
                    let distance_nm_times_64 = match distance_nm_times_64 {
                        Some(value) => {
                            value
                        }
                        None => {
                            print!("Time of flight too large; would overflow");
                            return;
                        }
                    };

                    let distance_mm = distance_nm_times_64 / 64 / 1_000_000;

                    print!("{:04x}:{:04x} - {} mm\n",
                        message.frame.header.source.pan_id,
                        message.frame.header.source.short_addr,
                        distance_mm,
                    );

                    return;
                }

                print!("Ignored message that was neither ping nor response\n");
            },
            |_error| {
                // ignore error
            },
        );
    }
}
