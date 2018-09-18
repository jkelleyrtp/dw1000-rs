//! Modifies a field in a DW1000 register and verifies that this worked


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

    write!(stdout, "Initializing...\n");

    // Initialize PANADR, so we can test `modify` in a controlled environment
    dwm1001.DW1000
        .panadr()
        .write(|w|
            w
                .short_addr(0x1234)
                .pan_id(0xabcd)
        )
        .expect("Failed to write to register");

    write!(stdout, "Modifying...\n");

    dwm1001.DW1000
        .panadr()
        .modify(|r, w| {
            assert_eq!(r.short_addr(), 0x1234);
            assert_eq!(r.pan_id(),     0xabcd);

            w.pan_id(0x5a5a)
        })
        .expect("Failed to modify register");

    write!(stdout, "Reading...\n");

    let panadr = dwm1001.DW1000
        .panadr()
        .read()
        .expect("Failed to read from register");

    assert_eq!(panadr.short_addr(), 0x1234);
    assert_eq!(panadr.pan_id(),     0x5a5a);

    write!(stdout, "Success!\n");

    loop {}
}
