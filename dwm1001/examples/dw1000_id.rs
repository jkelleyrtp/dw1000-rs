//! Establishes communication with the DW1000 and verifies its identity
//!
//! This example establishes SPI communication with the DW1000, reads its DEV_ID
//! register, and verifies that all its fields are as expected.

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();

    let dev_id = dwm1001
        .DW1000
        .ll()
        .dev_id()
        .read()
        .expect("Failed to read DEV_ID register");

    assert_eq!(dev_id.ridtag(), 0xDECA);
    assert_eq!(dev_id.model(), 0x01);
    assert_eq!(dev_id.ver(), 0x3);
    assert_eq!(dev_id.rev(), 0x0);

    defmt::info!("Success!");

    loop {}
}
