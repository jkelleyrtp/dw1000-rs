//! Accesses the LIS2DH12 3-axis accelerometer
//!
//! The example use the lis2dh2 driver
//! [lis2dh12](https://crates.io/crates/lis2dh12)
//! The driver implements the `Accelerometer` triat
//! [Accelerometer](https://crates.io/crates/accelerometer)

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use lis2dh12::RawAccelerometer;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("start");

    let dwm1001 = dwm1001::DWM1001::take().unwrap();

    // SDO/SA0 is connected to the supply voltage, so the least significant bit
    // is 1. See datasheet, section 6.1.1.

    let address = lis2dh12::SlaveAddr::Alternative(true);

    let mut lis2dh12 =
        lis2dh12::Lis2dh12::new(dwm1001.LIS2DH12, address).expect("lis2dh12 new failed");

    defmt::info!(
        "WHOAMI: {:08b}",
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

    defmt::info!(
        "TEMP: {:?}",
        lis2dh12.get_temp_out().expect("lis2dh2 get_temp failed")
    );

    defmt::info!(
        "TEMP_STATUS: {:?}",
        lis2dh12
            .get_temp_status()
            .expect("lis2dh2 get_temp_status failed")
    );

    defmt::info!("STATUS: {:?}", defmt::Debug2Format(&lis2dh12.get_status()));

    loop {
        defmt::info!("ACC: {:?}", defmt::Debug2Format(&lis2dh12.accel_raw()));
    }
}
