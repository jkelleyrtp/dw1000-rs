#![feature(panic_implementation)]


#![no_main]
#![no_std]


#[macro_use]
extern crate cortex_m_rt;

extern crate dwm1001;


use core::panic::PanicInfo;

use dwm1001::{
    cortex_m_rt::ExceptionFrame,
    nrf52_hal::{
        prelude::*,
        gpio::GpioExt,
        nrf52::{
            self,
            Peripherals,
        },
    },
};


entry!(main);

fn main() -> ! {
    let mut p = Peripherals::take().unwrap();

    let mut p0_14 = p.P0
        .split()
        .p0_14
        .into_push_pull_output();

    // Configure TIMER0
    p.TIMER0.shorts.write(|w|
        w
            .compare0_clear().enabled()
            .compare0_stop().enabled()
    );
    p.TIMER0.prescaler.write(|w|
        unsafe { w.prescaler().bits(5) } // 1 MHz
    );
    p.TIMER0.bitmode.write(|w|
        w.bitmode()._32bit()
    );

    loop {
        // Set P0.14 to LOW, thereby enabling the LED
        p0_14.set_low();

        delay(&mut p.TIMER0, 20_000); // 20ms

        // Set P0.14 to HIGH, thereby disabling the LED
        p0_14.set_high();

        delay(&mut p.TIMER0, 230_000); // 230ms
    }
}


fn delay(timer: &mut nrf52::TIMER0, cycles: u32) {
    // Configure timer to trigger EVENTS_COMPARE on number of cycles reached
    timer.cc[0].write(|w|
        // The timer was set to 32 bits, so all values of `cycles` are valid.
        unsafe { w.cc().bits(cycles) }
    );

    // Start timer
    timer.tasks_start.write(|w|
        unsafe { w.bits(1) }
    );

    // Wait for timer to reach the desired value
    while timer.events_compare[0].read().bits() == 0 {}

    // Reset the event
    timer.events_compare[0].write(|w| w);
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
