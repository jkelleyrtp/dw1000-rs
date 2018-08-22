//! Establishes communication with the DW1000 and verifies its identity
//!
//! This example establishes SPI communication with the DW1000, reads its DEV_ID
//! register, and verifies that all its fields are as expected.
//!
//! If everything is okay, it will blink once every second. If the contents of
//! DEV_ID are not what we expect, it starts a fast blinking pattern.


#![no_main]
#![no_std]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate nb;

extern crate cortex_m_semihosting;
extern crate dwm1001;
extern crate panic_semihosting;


use core::fmt::Write;

use cortex_m_semihosting::hio;
use dwm1001::{
    nrf52_hal::{
        prelude::*,
        timer::Timer,
    },
    DWM1001,
};


entry!(main);

fn main() -> ! {
    // Initialize semihosting for debug output
    let mut stdout = hio::hstdout()
        .expect("Failed to initialize semihosting");

    let mut dwm1001 = DWM1001::take().unwrap();

    let dev_id = dwm1001.DW1000.dev_id()
        .expect("Failed to read DEV_ID register");

    let is_as_expected =
        dev_id.ridtag() == 0xDECA &&
        dev_id.model()  == 0x01 &&
        dev_id.ver()    == 0x3 &&
        dev_id.rev()    == 0x0;

    // If everything is as expected, blink slow. Else, blink fast.
    let (low, high) = if is_as_expected {
        writeln!(stdout, "Success!")
            .expect("Failed to write to stdout");
        (30_000, 970_000)
    }
    else {
        writeln!(stdout, "Failure!")
            .expect("Failed to write to stdout");
        (100_000, 100_000)
    };

    // Configure timer and status LED
    let mut timer = dwm1001.TIMER0.constrain();
    let mut p0_14 = dwm1001.pins.p0_14.into_push_pull_output();

    loop {
        p0_14.set_low();
        delay(&mut timer, low);

        p0_14.set_high();
        delay(&mut timer, high);
    }
}


fn delay<T>(timer: &mut Timer<T>, cycles: u32) where T: TimerExt {
    timer.start(cycles);
    block!(timer.wait());
}
