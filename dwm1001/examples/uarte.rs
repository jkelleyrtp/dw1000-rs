#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

use nb::block;

use dwm1001::nrf52832_hal::{
    prelude::*,
    timer::{self, Timer},
};

#[cortex_m_rt::entry]
fn main() -> ! {
    let mut dwm1001 = dwm1001::DWM1001::take().unwrap();

    let mut timer = Timer::new(dwm1001.TIMER0);

    let out: [u8; 6] = [
        0x70, // P
        0x69, // I
        0x6e, // N
        0x67, // G
        0x0d, // CR
        0x0a, // LF
    ];

    loop {
        dwm1001.leds.D12.enable();
        delay(&mut timer, 20_000); // 20ms
        dwm1001.leds.D12.disable();
        delay(&mut timer, 230_000); // 230ms

        dwm1001.uart.write(&out).unwrap();
    }
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: timer::Instance,
{
    timer.start(cycles);
    block!(timer.wait()).unwrap();
}
