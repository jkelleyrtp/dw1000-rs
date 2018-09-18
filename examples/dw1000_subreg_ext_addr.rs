//! Accesses a DW1000 sub-register and verifies that this worked
//!
//! The main difference between this example and `dw1000_subreg.rs` is that
//! we're accessing a register with an extended address here, which means we use
//! a 3-byte header.


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
        .lde_cfg2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x1607)
        )
        .expect("Failed to write to register");

    write!(stdout, "Reading...\n");

    let lde_cfg2 = dwm1001.DW1000
        .lde_cfg2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(lde_cfg2.value(), 0x1607);

    write!(stdout, "Writing...\n");

    dwm1001.DW1000
        .lde_cfg2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x0607)
        )
        .expect("Failed to write to register");

    write!(stdout, "Reading...\n");

    let lde_cfg2 = dwm1001.DW1000
        .lde_cfg2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(lde_cfg2.value(), 0x0607);

    write!(stdout, "Success!\n");

    loop {}
}
