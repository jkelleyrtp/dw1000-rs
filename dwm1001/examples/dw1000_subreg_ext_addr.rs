//! Accesses a DW1000 sub-register and verifies that this worked
//!
//! The main difference between this example and `dw1000_subreg.rs` is that
//! we're accessing a register with an extended address here, which means we use
//! a 3-byte header.

#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_rt::entry;

use dwm1001::{debug, print, DWM1001};

#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    print!("Writing...\n");

    dwm1001
        .DW1000
        .ll()
        .lde_cfg2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x1607))
        .expect("Failed to write to register");

    print!("Reading...\n");

    let lde_cfg2 = dwm1001
        .DW1000
        .ll()
        .lde_cfg2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(lde_cfg2.value(), 0x1607);

    print!("Writing...\n");

    dwm1001
        .DW1000
        .ll()
        .lde_cfg2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x0607))
        .expect("Failed to write to register");

    print!("Reading...\n");

    let lde_cfg2 = dwm1001
        .DW1000
        .ll()
        .lde_cfg2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(lde_cfg2.value(), 0x0607);

    print!("Success!\n");

    loop {}
}
