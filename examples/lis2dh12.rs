//! Accesses the LIS2DH12 3-axis accelerometer
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_semihosting;

use dwm1001::{
    debug,
    DWM1001,
    print,
};


#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    // SDO/SA0 is connected to the supply voltage, so the least significant bit
    // is 1. See datasheet, section 6.1.1.
    let address = 0b0011001;

    let mut write_buf = [0u8; 1];
    let mut read_buf  = [0u8; 1];

    // Read WHOAMI register (address 0x0F)
    write_buf[0] = 0x0F;
    dwm1001.LIS2DH12.write(address, &write_buf)
        .expect("Failed to write to I2C");
    dwm1001.LIS2DH12.read(address, &mut read_buf)
        .expect("Failed to read from I2C");
    assert_eq!(read_buf[0], 0b00110011);

    print!("WHOAMI: {:08b}\n", read_buf[0]);

    loop {}
}
