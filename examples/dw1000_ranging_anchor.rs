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

    // These are the hardcoded calibration values from the dwm1001-examples
    // repository[1]. Ideally, the calibration values would be determined using
    // the proper calibration procedure, but hopefully those are good enough for
    // now.
    //
    // [1] https://github.com/Decawave/dwm1001-examples
    dw1000.set_antenna_delay(16456, 16300)
        .expect("Failed to set antenna delay");

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

        // Listen for ranging requests
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
                let request = ranging::Request::decode(&message);

                let request = match request {
                    Ok(Some(request)) =>
                        request,
                    Ok(None) | Err(_) => {
                        print!("Ignoring message that is not a request\n");
                        return;
                    }
                };

                // Send ranging response
                let mut future = ranging::Response
                    ::initiate(&mut dw1000, request)
                    .expect("Failed to initiate response")
                    .send(&mut dw1000)
                    .expect("Failed to initiate response transmission");
                block!(future.wait())
                    .expect("Failed to send ranging response");
            },
            |_error| {
                // ignore
            },
        );

        // After receiving for a while, it's time to send out a ping
        let mut future = ranging::Ping
            ::initiate(&mut dw1000)
            .expect("Failed to initiate ping")
            .send(&mut dw1000)
            .expect("Failed to initiate ping transmission");
        block!(future.wait())
            .expect("Failed to send ping");
    }
}
