//! Writes to a DW1000 register, reads it back, and verifies its value
//!
//! This is a basic test of the DW1000 driver's register access code.

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
        .tx_fctrl()
        .write(|w| {
            w.tflen(0b100_1001)
                .tfle(0b10_1)
                .txbr(0b10)
                .tr(0b1)
                .txprf(0b01)
                .txpsr(0b01)
                .pe(0b10)
                .txboffs(0b1101_0010_11)
                .ifsdelay(0b0110_0110)
        })
        .expect("Failed to write to register");

    defmt::info!("Reading...\n");

    let tx_fctrl = dwm1001
        .DW1000
        .ll()
        .tx_fctrl()
        .read()
        .expect("Failed to read from register");

    assert_eq!(tx_fctrl.tflen(), 0b1001001);
    assert_eq!(tx_fctrl.tfle(), 0b101);
    assert_eq!(tx_fctrl.txbr(), 0b10);
    assert_eq!(tx_fctrl.tr(), 0b1);
    assert_eq!(tx_fctrl.txprf(), 0b01);
    assert_eq!(tx_fctrl.txpsr(), 0b01);
    assert_eq!(tx_fctrl.pe(), 0b10);
    assert_eq!(tx_fctrl.txboffs(), 0b1101001011);
    assert_eq!(tx_fctrl.ifsdelay(), 0b01100110);

    defmt::info!("Success!\n");

    loop {}
}
