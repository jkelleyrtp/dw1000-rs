#![no_main]
#![no_std]


extern crate panic_semihosting;


use core::fmt::Write;

use cortex_m_rt::entry;
use heapless::String as HString;
use nb::block;

use dwm1001::{
    nrf52832_hal::{prelude::*, timer::{self, Timer}},
    DWM1001,
};


#[entry]
fn main() -> ! {
    let mut dwm1001 = DWM1001::take().unwrap();

    let mut timer = Timer::new(dwm1001.TIMER0);

    let mut s: HString<64> = HString::new();
    s.push_str("halp plz ")
        .expect("Failed to push to string");
    let original_len = s.len();

    for i in 0.. {
        dwm1001.leds.D12.enable();
        delay(&mut timer, 20_000); // 20ms
        dwm1001.leds.D12.disable();
        delay(&mut timer, 230_000); // 230ms

        write!(s, "{}\r\n", i)
            .expect("Failed to write to string");

        dwm1001.uart.write(s.as_bytes()).unwrap();

        s.truncate(original_len);
    }
    
    // convince the compiler this never terminates
    loop {}
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: timer::Instance,
{
    timer.start(cycles);
    block!(timer.wait()).unwrap();
}
