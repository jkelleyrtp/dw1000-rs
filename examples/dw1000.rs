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
    cortex_m_rt::ExceptionFrame,
    nrf52_hal::{
        prelude::*,
        nrf52::Peripherals,
        timer::Timer,
    },
};


entry!(main);

fn main() -> ! {
    let p = Peripherals::take().unwrap();

    let pins = p.P0.split();
    let spim = p.SPIM0;

    // Select P0.16 for SCK
    let p0_16 = pins.p0_16
        .into_push_pull_output()
        .degrade();
    spim.psel.sck.write(|w| {
        let w = unsafe { w.pin().bits(p0_16.pin) };
        w.connect().connected()
    });

    // Select P0.20 for MOSI
    let p0_20 = pins.p0_20
        .into_push_pull_output()
        .degrade();
    spim.psel.mosi.write(|w| {
        let w = unsafe { w.pin().bits(p0_20.pin) };
        w.connect().connected()
    });

    // Select P0.18 for MISO
    let p0_18 = pins.p0_18
        .into_floating_input()
        .degrade();
    spim.psel.miso.write(|w| {
        let w = unsafe { w.pin().bits(p0_18.pin) };
        w.connect().connected()
    });

    // Use P0.17 as the chip select pin for the DW1000 (DW_CS)
    //
    // This is not controlled by the SPIM0 peripheral, so we need to use the
    // GPIO pin directly. This is initially pulled high, which is the inactive
    // state.
    let mut dw_cs = pins.p0_17.into_push_pull_output();
    dw_cs.set_high();

    // Enable SPIM0
    //
    // SPIM0 shares the same address space with SPIS0, SPI0, TWIM0, TWIS0, and
    // TWI0. All of those are disabled by default, so there's no problem here.
    spim.enable.write(|w|
        w.enable().enabled()
    );

    // Set SPIM0 to SPI mode 0
    //
    // The DW1000's SPI mode can be configured, but on the DWM1001 board, both
    // configuration pins (GPIO5/SPIPOL and GPIO6/SPIPHA) are unconnected and
    // internally pulled low, setting it to SPI mode 0.
    spim.config.write(|w|
        w
            .order().msb_first()
            .cpha().leading()
            .cpol().active_high()
    );

    // Configure frequency
    //
    // According to its documentation, the DW1000 should be able to handle up to
    // 3 MHz. Let's play it safe and choose something lower.
    spim.frequency.write(|w|
        w.frequency().k500() // 500 kHz
    );

    // Set over-read character to '0'
    //
    // This is the character that will be written out during a transaction, once
    // the TXD buffer is exhausted.
    spim.orc.write(|w|
        // The ORC field is 8 bits long, so `0` is a valid value to write there.
        unsafe { w.orc().bits(0) }
    );

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
    spim.txd.ptr.write(|w|
        // We're giving the register a pointer to the stack. Since we're waiting
        // for the SPI transaction to end before this stack pointer becomes
        // invalid, there's nothing wrong here.
        //
        // The PTR field is a full 32 bits wide and accepts arbitrary values.
        unsafe { w.ptr().bits(txd_buffer.as_ptr() as u32) }
    );
    spim.txd.maxcnt.write(|w|
        // We're giving it the length of the buffer, so no danger of accessing
        // invalid memory. We know the length of the buffer to be exactly `1`,
        // so the cast to `u8` is also fine.
        //
        // The MAXCNT is 8 bits wide and the full range of values, so even if we
        // didn't know the actual value, writing any `u8` would be fine.
        unsafe { w.maxcnt().bits(txd_buffer.len() as u8) }
    );

    // Set up the RXD buffer
    //
    // SPI is a synchronous interface, so we're going to receive a byte for
    // every one we send. That means in addition to the 4 bytes we actually
    // expect, we need an additional one that we receive while we send the
    // header.
    let mut rxd_buffer = [0u8; 5];
    spim.rxd.ptr.write(|w|
        // This is safe for the same reasons that writing TXD.PTR is safe.
        // Please refer to the explanation there.
        unsafe { w.ptr().bits(rxd_buffer.as_mut_ptr() as u32) }
    );
    spim.rxd.maxcnt.write(|w|
        // This is safe for the same reasons that writing TXD.MAXCNT is safe.
        // Please refer to the explanation there.
        unsafe { w.maxcnt().bits(rxd_buffer.len() as u8) }
    );

    // Start SPI transaction
    dw_cs.set_low();
    spim.tasks_start.write(|w|
        // `1` is a valid value to write to task registers.
        unsafe { w.bits(1) }
    );

    // Wait for END event
    //
    // This event is triggered once both transmitting and receiving are done.
    while spim.events_end.read().bits() == 0 {}

    // End SPI transaction
    dw_cs.set_high();

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


exception!(*, default_handler);
exception!(HardFault, handle_hard_fault);

fn default_handler(_irqn: i16) {
    loop {}
}

fn handle_hard_fault(_ef: &ExceptionFrame) -> ! {
    loop {}
}
