//! Accesses a DW1000 sub-register and verifies that this worked

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();

    defmt::info!("Writing...");

    dwm1001
        .DW1000
        .ll()
        .drx_tune2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x311A002D))
        .expect("Failed to write to register");

    defmt::info!("Reading...");

    let drx_tune2 = dwm1001
        .DW1000
        .ll()
        .drx_tune2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(drx_tune2.value(), 0x311A002D);

    defmt::info!("Writing...");

    dwm1001
        .DW1000
        .ll()
        .drx_tune2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x313B006B))
        .expect("Failed to write to register");

    defmt::info!("Reading...");

    let drx_tune2 = dwm1001
        .DW1000
        .ll()
        .drx_tune2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(drx_tune2.value(), 0x313B006B);

    defmt::info!("Success!");

    loop {}
}
