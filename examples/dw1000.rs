//! Establishes communication with the DW1000 and verifies its identity
//!
//! This example establishes SPI communication with the DW1000, reads its DEV_ID
//! register, and verifies that all its fields are as expected.
//!
//! If everything is okay, it will blink once every second. If the contents of
//! DEV_ID are not what we expect, it starts a fast blinking pattern.


#![feature(panic_implementation)]


#![no_main]
#![no_std]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate nb;

extern crate dwm1001;


use core::panic::PanicInfo;

use dwm1001::{
    nrf52_hal::{
        prelude::*,
        nrf52::Peripherals,
        spim,
        timer::Timer,
    },
};


entry!(main);

fn main() -> ! {
    let p = Peripherals::take().unwrap();

    let pins = p.P0.split();

    let spi_pins = spim::Pins {
        sck : pins.p0_16.into_push_pull_output().degrade(),
        mosi: pins.p0_20.into_push_pull_output().degrade(),
        miso: pins.p0_18.into_floating_input().degrade(),
    };

    let mut dw_cs = pins.p0_17.into_push_pull_output().degrade();

    // Some notes about the hardcoded configuration of `Spim`:
    // - The DW1000's SPI mode can be configured, but on the DWM1001 board, both
    //   configuration pins (GPIO5/SPIPOL and GPIO6/SPIPHA) are unconnected and
    //   internally pulled low, setting it to SPI mode 0.
    // - The frequency is set to a moderate value that the DW1000 can easily
    //   handle.
    let mut spim = p.SPIM0.constrain(spi_pins);

    // Set up the TXD buffer
    //
    // It consists of only one byte for the transaction header. Since this is a
    // read operation, there is no transaction body.
    //
    // The transaction signals a read without a sub-index, which means it's 1
    // byte long. This byte consists of the following bits:
    //   7: 0 for read
    //   6: 0 for no sub-index
    // 5-0: 0 for DEV_ID register
    let txd_buffer = [0u8];

    // Set up the RXD buffer
    //
    // SPI is a synchronous interface, so we're going to receive a byte for
    // every one we send. That means in addition to the 4 bytes we actually
    // expect, we need an additional one that we receive while we send the
    // header.
    let mut rxd_buffer = [0u8; 5];

    spim.read(&mut dw_cs, &txd_buffer, &mut rxd_buffer)
        .expect("Failed to read from DW1000");

    // Extract the fields of the DEV_ID register that we read
    let ridtag = (rxd_buffer[4] as u16) << 8 | rxd_buffer[3] as u16;
    let model  = rxd_buffer[2];
    let ver    = (rxd_buffer[1] & 0xf0) >> 4;
    let rev    = rxd_buffer[1] & 0x0f;

    let is_as_expected =
        ridtag == 0xDECA &&
        model  == 0x01 &&
        ver    == 0x3 &&
        rev    == 0x0;

    // If everything is as expected, blink slow. Else, blink fast.
    let (low, high) = if is_as_expected {
        (30_000, 970_000)
    }
    else {
        (100_000, 100_000)
    };

    // Configure timer and status LED
    let mut timer = p.TIMER0.constrain();
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


#[panic_implementation]
#[no_mangle]
pub fn panic(_: &PanicInfo) -> ! {
    loop {}
}
