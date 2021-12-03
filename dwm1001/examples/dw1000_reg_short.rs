//! Writes to and reads from a DW1000 that is shorter than its representation
//!
//! Some registers are shorter than some of the types that represent their
//! fields. For example, a register migth be 40 bits wide, but have a field that
//! is represented by a `u64`. This example makes sure to exercise the register
//! read/write infrastructure using such a field.

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();

    defmt::info!("Writing...\n");

    dwm1001
        .DW1000
        .ll()
        .dx_time()
        .write(|w| w.value(0x1122334455))
        .expect("Failed to write to register");

    defmt::info!("Reading...\n");

    let dx_time = dwm1001
        .DW1000
        .ll()
        .dx_time()
        .read()
        .expect("Failed to read from register");

    assert_eq!(dx_time.value(), 0x1122334455);

    defmt::info!("Success!\n");

    loop {}
}
