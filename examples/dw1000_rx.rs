//! Waits to receive data and signals status via LEDs
//!
//! Please note that, for unknown reasons, this example only receives data once,
//! then needs to be re-flashed to receive data again. It's a start.


#![no_main]
#![no_std]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate dwm1001;

extern crate cortex_m_semihosting;
extern crate nb;
extern crate panic_semihosting;


use dwm1001::{
    prelude::*,
    debug,
    nrf52832_hal::Delay,
    DWM1001,
};





#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    let     clocks = dwm1001.CLOCK.constrain().freeze();
    let mut delay  = Delay::new(dwm1001.SYST, clocks);

    dwm1001.DW_RST.reset_dw1000(&mut delay);

    // Configure timer
    let mut timer = dwm1001.TIMER0.constrain();

    'outer: loop {
        dwm1001.DW1000
            .start_receiver()
            .expect("Failed to start receiver");

        // Set timer for timeout
        timer.start(5_000_000);

        // Wait until frame has been received
        loop {
            let sys_status = dwm1001.DW1000
                .ll()
                .sys_status()
                .read()
                .expect("Failed to read from register");

            // Check progress
            if sys_status.rxprd() == 0b1 {
                dwm1001.DW1000
                    .ll()
                    .sys_status()
                    .write(|w| w.rxprd(0b1))
                    .expect("Failed to reset flag");
                print!("Preamble detected\n");
            }
            if sys_status.rxsfdd() == 0b1 {
                dwm1001.DW1000
                    .ll()
                    .sys_status()
                    .write(|w| w.rxsfdd(0b1))
                    .expect("Failed to reset flag");
                print!("SFD detected\n");
            }
            if sys_status.rxphd() == 0b1 {
                dwm1001.DW1000
                    .ll()
                    .sys_status()
                    .write(|w| w.rxphd(0b1))
                    .expect("Failed to reset flag");
                print!("PHY header detected\n");
            }
            if sys_status.rxdfr() == 0b1 {
                dwm1001.DW1000
                    .ll()
                    .sys_status()
                    .write(|w| w.rxdfr(0b1))
                    .expect("Failed to reset flag");
                print!("Data frame ready\n");

                // Check for errors
                if sys_status.rxfce() == 0b1 {
                    print!("FCS error\n");
                    continue 'outer;
                }
                if sys_status.rxfcg() != 0b1 {
                    print!("FCS not good\n");
                    continue 'outer;
                }

                break;
            }

            // Check errors
            if sys_status.ldeerr() == 0b1 {
                dwm1001.DW1000
                    .ll()
                    .sys_status()
                    .write(|w| w.ldeerr(0b1))
                    .expect("Failed to reset flag");
                print!("Leading edge detection error\n");
            }
            if sys_status.rxprej() == 0b1 {
                dwm1001.DW1000
                    .ll()
                    .sys_status()
                    .write(|w| w.rxprej(0b1))
                    .expect("Failed to reset flag");
                print!("Preamble rejection\n");
            }
            if sys_status.rxphe() == 0b1 {
                dwm1001.DW1000
                    .ll()
                    .sys_status()
                    .write(|w| w.rxphe(0b1))
                    .expect("Failed to reset flag");
                print!("PHY header error\n");

                continue 'outer;
            }

            if timer.wait() != Err(nb::Error::WouldBlock) {
                print!("Timeout\n");
                continue 'outer;
            }
        }

        print!("Process...\n");

        // Read received frame
        let rx_finfo = dwm1001.DW1000
            .ll()
            .rx_finfo()
            .read()
            .expect("Failed to read from register");
        let rx_buffer = dwm1001.DW1000
            .ll()
            .rx_buffer()
            .read()
            .expect("Failed to read from register");

        let len  = rx_finfo.rxflen() as usize;
        let data = &rx_buffer.data()[0 .. len];

        print!("Received data: {:?}\n", data);

        let expected_data = b"ping";

        // Received data should have length of expected data, plus 2-byte CRC
        // checksum.
        if len != expected_data.len() + 2 {
            print!("Unexpected length: {}\n", len);
            continue;
        }

        if data[0 .. len - 2] != expected_data[..] {
            print!("Unexpected data");
            continue;
        }

        // Signal that data was received
        for _ in 0..20 {
            dwm1001.leds.D10.enable();
            delay.delay_ms(30u32);
            dwm1001.leds.D10.disable();
            delay.delay_ms(30u32);
        }
    }
}
