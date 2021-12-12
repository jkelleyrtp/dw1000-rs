//! Modifies a field in a DW1000 register and verifies that this worked

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();

    defmt::info!("Initializing...\n");

    // Initialize PANADR, so we can test `modify` in a controlled environment
    dwm1001
        .DW1000
        .ll()
        .panadr()
        .write(|w| w.short_addr(0x1234).pan_id(0xabcd))
        .expect("Failed to write to register");

    defmt::info!("Modifying...\n");

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

    defmt::info!("Reading...\n");

    let panadr = dwm1001
        .DW1000
        .ll()
        .panadr()
        .read()
        .expect("Failed to read from register");

    assert_eq!(panadr.short_addr(), 0x1234);
    assert_eq!(panadr.pan_id(), 0x5a5a);

    defmt::info!("Success!\n");

    loop {}
}
