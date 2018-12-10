#![no_main]
#![no_std]

use cortex_m_rt::entry;
use nb::block;
use panic_semihosting;

use dwm1001::{
    nrf52832_hal::{
        prelude::*,
        timer::Timer,
    },
    DWM1001,
};


#[entry]
fn main() -> ! {
    let mut dwm1001 = DWM1001::take().unwrap();

    let mut timer = dwm1001.TIMER0.constrain();

    let out: [u8; 6] = [
        0x70, // P
        0x69, // I
        0x6e, // N
        0x67, // G
        0x0d, // CR
        0x0a  // LF
    ];

    loop {
        dwm1001.leds.D12.enable();
        delay(&mut timer, 20_000); // 20ms
        dwm1001.leds.D12.disable();
        delay(&mut timer, 230_000); // 230ms

        dwm1001.uart.write(&out).unwrap();
    }
}


fn delay<T>(timer: &mut Timer<T>, cycles: u32) where T: TimerExt {
    timer.start(cycles);
    block!(timer.wait()).unwrap();
}
