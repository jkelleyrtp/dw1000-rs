//! Accesses a DW1000 sub-register and verifies that this worked
//!
//! The main difference between this example and `dw1000_subreg.rs` is that
//! we're accessing a register with an extended address here, which means we use
//! a 3-byte header.

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
        .lde_cfg2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x1607))
        .expect("Failed to write to register");

    defmt::info!("Reading...\n");

    let lde_cfg2 = dwm1001
        .DW1000
        .ll()
        .lde_cfg2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(lde_cfg2.value(), 0x1607);

    defmt::info!("Writing...\n");

    dwm1001
        .DW1000
        .ll()
        .lde_cfg2()
        .write(|w|
            // Careful, only specific values are allowed here.
            w.value(0x0607))
        .expect("Failed to write to register");

    defmt::info!("Reading...\n");

    let lde_cfg2 = dwm1001
        .DW1000
        .ll()
        .lde_cfg2()
        .read()
        .expect("Failed to read from register");

    assert_eq!(lde_cfg2.value(), 0x0607);

    defmt::info!("Success!\n");

    loop {}
}
