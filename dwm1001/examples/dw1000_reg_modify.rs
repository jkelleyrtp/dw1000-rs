//! Modifies a field in a DW1000 register and verifies that this worked

#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_rt::entry;

use dwm1001::{debug, print, DWM1001};

#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    print!("Initializing...\n");

    // Initialize PANADR, so we can test `modify` in a controlled environment
    dwm1001
        .DW1000
        .ll()
        .panadr()
        .write(|w| w.short_addr(0x1234).pan_id(0xabcd))
        .expect("Failed to write to register");

    print!("Modifying...\n");

    dwm1001
        .DW1000
        .ll()
        .panadr()
        .modify(|r, w| {
            assert_eq!(r.short_addr(), 0x1234);
            assert_eq!(r.pan_id(), 0xabcd);

            w.pan_id(0x5a5a)
        })
        .expect("Failed to modify register");

    print!("Reading...\n");

    let panadr = dwm1001
        .DW1000
        .ll()
        .panadr()
        .read()
        .expect("Failed to read from register");

    assert_eq!(panadr.short_addr(), 0x1234);
    assert_eq!(panadr.pan_id(), 0x5a5a);

    print!("Success!\n");

    loop {}
}
