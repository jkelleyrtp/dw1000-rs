//! Writes to a DW1000 register, reads it back, and verifies its value
//!
//! This is a basic test of the DW1000 driver's register access code.


#![no_main]
#![no_std]


#[macro_use] extern crate cortex_m_rt;

extern crate cortex_m_semihosting;
extern crate dwm1001;
extern crate panic_semihosting;


use core::fmt::Write;

use cortex_m_semihosting::hio;

use dwm1001::{
    dw1000,
    DWM1001,
};


entry!(main);

fn main() -> ! {
    // Initialize debug output
    let mut stdout = hio::hstdout()
        .expect("Failed to initialize debug output");

    let mut dwm1001 = DWM1001::take().unwrap();

    write!(stdout, "Writing...\n");

    dwm1001.DW1000
        .write::<dw1000::TX_FCTRL, _>(|w|
            w
                .tflen(0b100_1001)
                .tfle(0b10_1)
                .txbr(0b10)
                .tr(0b1)
                .txprf(0b01)
                .txpsr(0b01)
                .pe(0b10)
                .txboffs(0b1101_0010_11)
                .ifsdelay(0b0110_0110)
        )
        .expect("Failed to write to register");

    write!(stdout, "Reading...\n");

    let tx_fctrl = dwm1001.DW1000.read::<dw1000::TX_FCTRL>()
        .expect("Failed to read from register");

    assert_eq!(tx_fctrl.tflen(),    0b1001001);
    assert_eq!(tx_fctrl.tfle(),     0b101);
    assert_eq!(tx_fctrl.txbr(),     0b10);
    assert_eq!(tx_fctrl.tr(),       0b1);
    assert_eq!(tx_fctrl.txprf(),    0b01);
    assert_eq!(tx_fctrl.txpsr(),    0b01);
    assert_eq!(tx_fctrl.pe(),       0b10);
    assert_eq!(tx_fctrl.txboffs(),  0b1101001011);
    assert_eq!(tx_fctrl.ifsdelay(), 0b01100110);

    write!(stdout, "Success!\n");

    loop {}
}
