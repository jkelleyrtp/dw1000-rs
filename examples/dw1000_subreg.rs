//! Accesses a DW1000 sub-register and verifies that this worked


#![no_main]
#![no_std]


#[macro_use] extern crate cortex_m_rt;

extern crate cortex_m_semihosting;
extern crate dwm1001;
extern crate panic_semihosting;


use core::fmt::Write;

use cortex_m_semihosting::hio;

use dwm1001::DWM1001;


#[entry]
fn main() -> ! {
    // Initialize debug output
    let mut stdout = hio::hstdout()
        .expect("Failed to initialize debug output");

    let mut dwm1001 = DWM1001::take().unwrap();

    write!(stdout, "Writing...\n");

    dwm1001.DW1000
        .drx_tune2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x311A002D)
        )
        .expect("Failed to write to register");

    write!(stdout, "Reading...\n");

    let drx_tune2 = dwm1001.DW1000
        .drx_tune2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(drx_tune2.value(), 0x311A002D);

    write!(stdout, "Writing...\n");

    dwm1001.DW1000
        .drx_tune2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x313B006B)
        )
        .expect("Failed to write to register");

    write!(stdout, "Reading...\n");

    let drx_tune2 = dwm1001.DW1000
        .drx_tune2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(drx_tune2.value(), 0x313B006B);

    write!(stdout, "Success!\n");

    loop {}
}
