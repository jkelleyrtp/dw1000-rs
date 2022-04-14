//! Continually sends data and signals its status via LEDs

#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    let dwm1001 = dwm1001::DWM1001::take().unwrap();
    let mut delay = nrf52832_hal::Delay::new(dwm1001.SYST);

    let mut dw1000 = dwm1001
        .DW1000
        .init(&mut delay)
        .unwrap()
        .with_rx_cfg(dw1000::RxConfig {
            frame_filtering: false,
            ..Default::default()
        });

    let mut buffer = [0; 1024];

    loop {
        let message = dw1000
            .receive()
            .expect("Failed to start receiver")
            .wait(&mut buffer);

        match nb::block!(message) {
            Ok(_message) => defmt::info!("A message was received!"),
            Err(_err) => defmt::info!("An error occurred while receiving a message"),
        }
    }
}
