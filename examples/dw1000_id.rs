//! Establishes communication with the DW1000 and verifies its identity
//!
//! This example establishes SPI communication with the DW1000, reads its DEV_ID
//! register, and verifies that all its fields are as expected.


#![no_main]
#![no_std]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate dwm1001;

extern crate panic_semihosting;


use dwm1001::{
    debug,
    DWM1001,
};


#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    let dev_id = dwm1001.DW1000
        .ll()
        .dev_id()
        .read()
        .expect("Failed to read DEV_ID register");

    assert_eq!(dev_id.ridtag(), 0xDECA);
    assert_eq!(dev_id.model(),  0x01);
    assert_eq!(dev_id.ver(),    0x3);
    assert_eq!(dev_id.rev(),    0x0);

    print!("Success!\n");

    loop {}
}
