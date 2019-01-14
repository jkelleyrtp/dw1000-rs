#![no_main]
#![no_std]

use core::fmt::Write;

use cortex_m_rt::entry;
use heapless::String as HString;
use nb::block;
use panic_semihosting;

use dwm1001::{
    nrf52832_hal::{prelude::*, timer::Timer},
    DWM1001,
};

#[entry]
fn main() -> ! {
    let mut dwm1001 = DWM1001::take().unwrap();

    let mut timer = dwm1001.TIMER0.constrain();

    let mut s: HString<heapless::consts::U64> = HString::new();
    s.push_str("halp plz ");
    let original_len = s.len();

    for i in 0.. {
        dwm1001.leds.D12.enable();
        delay(&mut timer, 20_000); // 20ms
        dwm1001.leds.D12.disable();
        delay(&mut timer, 230_000); // 230ms

        write!(s, "{}\r\n", i);

        dwm1001.uart.write(s.as_bytes()).unwrap();

        s.truncate(original_len);
    }
    
    // convince the compiler this never terminates
    loop {}
}

fn delay<T>(timer: &mut Timer<T>, cycles: u32)
where
    T: TimerExt,
{
    timer.start(cycles);
    block!(timer.wait()).unwrap();
}
