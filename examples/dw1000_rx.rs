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
        print!("Configure...\n");

        // For unknown reasons, the DW1000 get stuck in RX mode without ever
        // receiving anything, after receiving one good frame. Reset the
        // receiver to make sure its in a valid state before attempting to
        // receive anything.
        dwm1001.DW1000
            .ll()
            .pmsc_ctrl0()
            .modify(|_, w|
                w.softreset(0b1110) // reset receiver
            )
            .expect("Failed to modify register");
        dwm1001.DW1000
            .ll()
            .pmsc_ctrl0()
            .modify(|_, w|
                w.softreset(0b1111) // clear reset
            )
            .expect("Failed to modify register");

        // Set PLLLDT bit in EC_CTRL. According to the documentation of the
        // CLKPLL_LL bit in SYS_STATUS, this bit needs to be set to ensure the
        // reliable operation of the CLKPLL_LL bit. Since I've seen that bit
        // being set, I want to make sure I'm not just seeing crap.
        dwm1001.DW1000
            .ll()
            .ec_ctrl()
            .modify(|_, w|
                w.pllldt(0b1)
            )
            .expect("Failed to modify register");

        // Now that PLLLDT is set, clear all bits in SYS_STATUS that depend on
        // it for reliable operation. After that is done, these bits should work
        // reliably.
        dwm1001.DW1000
            .ll()
            .sys_status()
            .write(|w|
                w
                    .cplock(0b1)
                    .clkpll_ll(0b1)
            )
            .expect("Failed to write to register");

        // If we cared about MAC addresses, which we don't in this example, we'd
        // have to enable frame filtering at this point. By default it's off,
        // meaning we'll receive everything, no matter who it is addressed to.

        // We're expecting a preamble length of `64`. Set PAC size to the
        // recommended value for that preamble length, according to section
        // 4.1.1. The value we're writing to DRX_TUNE2 here also depends on the
        // PRF, which we expect to be 16 MHz.
        dwm1001.DW1000
            .ll()
            .drx_tune2()
            .write(|w|
                // PAC size 8, with 16 MHz PRF
                w.value(0x311A002D)
            )
            .expect("Failed to write to register");

        // If we were going to receive at 110 kbps, we'd need to set the RXM110K
        // bit in the System Configuration register. We're expecting to receive
        // at 850 kbps though, so the default is fine. See section 4.1.3 for a
        // detailed explanation.

        print!("Receive...\n");

        dwm1001.DW1000
            .ll()
            .sys_ctrl()
            .modify(|_, w|
                w.rxenab(0b1)
            )
            .expect("Failed to modify register");

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

                continue 'outer;
            }
            if sys_status.rxprej() == 0b1 {
                dwm1001.DW1000
                    .ll()
                    .sys_status()
                    .write(|w| w.rxprej(0b1))
                    .expect("Failed to reset flag");
                print!("Preamble rejection\n");

                continue 'outer;
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
