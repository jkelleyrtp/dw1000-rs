//! Accesses the LIS2DH12 3-axis accelerometer
//!
//! The example use the lis2dh2 driver
//! [lis2dh12](https://crates.io/crates/lis2dh12)
//! The driver implements the `Accelerometer` triat
//! [Accelerometer](https://crates.io/crates/accelerometer)

#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_rt::entry;

use dwm1001::{debug, print, DWM1001};

use lis2dh12::{self, RawAccelerometer};

#[entry]
fn main() -> ! {
    debug::init();

    print!("start\n");

    let dwm1001 = DWM1001::take().unwrap();

    // SDO/SA0 is connected to the supply voltage, so the least significant bit
    // is 1. See datasheet, section 6.1.1.

    let address = lis2dh12::SlaveAddr::Alternative(true);

    let mut lis2dh12 =
        lis2dh12::Lis2dh12::new(dwm1001.LIS2DH12, address).expect("lis2dh12 new failed");

    print!(
        "WHOAMI: {:08b}\n",
        lis2dh12
            .get_device_id()
            .expect("lis2dh12 get_device_id failed")
    );

    lis2dh12
        .set_mode(lis2dh12::Mode::HighResolution)
        .expect("lis2dh12 set_mode failed");

    lis2dh12
        .set_odr(lis2dh12::Odr::Hz1)
        .expect("lis2dh12 set_odr failed");

    lis2dh12
        .enable_axis((true, true, true))
        .expect("lis2dh12 enable_axis failed");

    lis2dh12
        .enable_temp(true)
        .expect("lis2dh2 enable_temp failed");

    print!(
        "TEMP: {:?}\n",
        lis2dh12.get_temp_out().expect("lis2dh2 get_temp failed")
    );

    print!(
        "TEMP_STATUS: {:?}\n",
        lis2dh12
            .get_temp_status()
            .expect("lis2dh2 get_temp_status failed")
    );

    print!("STATUS: {:?}\n", lis2dh12.get_status());

    loop {
        print!("ACC: {:?}\n", lis2dh12.accel_raw());
    }
}
