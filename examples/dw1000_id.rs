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
    dw1000::DW1000,
    nrf52_hal::{
        prelude::*,
        spim,
        timer::Timer,
    },
    DWM1001,
};


entry!(main);

fn main() -> ! {
    // Initialize semihosting for debug output
    let mut stdout = hio::hstdout()
        .expect("Failed to initialize semihosting");

    let dwm1001 = DWM1001::take().unwrap();

    let pins = dwm1001.P0.split();

    // Some notes about the hardcoded configuration of `Spim`:
    // - The DW1000's SPI mode can be configured, but on the DWM1001 board, both
    //   configuration pins (GPIO5/SPIPOL and GPIO6/SPIPHA) are unconnected and
    //   internally pulled low, setting it to SPI mode 0.
    // - The frequency is set to a moderate value that the DW1000 can easily
    //   handle.
    let spim = dwm1001.SPIM2.constrain(spim::Pins {
        sck : pins.p0_16.into_push_pull_output().degrade(),
        mosi: pins.p0_20.into_push_pull_output().degrade(),
        miso: pins.p0_18.into_floating_input().degrade(),
    });

    let dw_cs = pins.p0_17.into_push_pull_output().degrade();

    let mut dw1000 = DW1000::new(spim, dw_cs);

    let dev_id = dw1000.dev_id()
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
    let mut p0_14 = pins.p0_14.into_push_pull_output();

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
