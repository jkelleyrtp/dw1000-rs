//! Accesses a DW1000 sub-register and verifies that this worked
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_semihosting;

use dwm1001::{
    debug,
    DWM1001,
    print,
};


#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    print!("Writing...\n");

    dwm1001.DW1000
        .ll()
        .drx_tune2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x311A002D)
        )
        .expect("Failed to write to register");

    print!("Reading...\n");

    let drx_tune2 = dwm1001.DW1000
        .ll()
        .drx_tune2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(drx_tune2.value(), 0x311A002D);

    print!("Writing...\n");

    dwm1001.DW1000
        .ll()
        .drx_tune2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x313B006B)
        )
        .expect("Failed to write to register");

    print!("Reading...\n");

    let drx_tune2 = dwm1001.DW1000
        .ll()
        .drx_tune2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(drx_tune2.value(), 0x313B006B);

    print!("Success!\n");

    loop {}
}
