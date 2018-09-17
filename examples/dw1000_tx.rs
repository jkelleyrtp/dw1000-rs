//! Continually sends data and signals its status via LEDs


#![no_main]
#![no_std]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate dwm1001;

extern crate cortex_m_semihosting;
extern crate panic_semihosting;


use dwm1001::{
    debug,
    dw1000,
    DWM1001,
};


entry!(main);

fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    loop {
        let tx_data = b"ping";

        dwm1001.leds.D9.enable();
        print!("Configure...\n");

        // Prepare transmitter
        dwm1001.DW1000
            .write::<dw1000::TX_BUFFER, _>(|w|
                w.data(tx_data)
            )
            .expect("Failed to write to register");
        dwm1001.DW1000
            .write::<dw1000::TX_FCTRL, _>(|w| {
                let tflen = tx_data.len() as u8 + 2;
                w
                    .tflen(tflen) // data length + two-octet CRC
                    .tfle(0)      // no non-standard length extension
                    .txbr(0b01)   // 850 kbps bit rate
                    .tr(0b0)      // no ranging
                    .txprf(0b01)  // pulse repetition frequency: 16 MHz
                    .txpsr(0b01)  // preamble length: 64
                    .pe(0b00)     // no non-standard preamble length
                    .txboffs(0)   // no offset in TX_BUFFER
                    .ifsdelay(0)  // no delay between frames
            })
            .expect("Failed to write to register");

        dwm1001.leds.D9.disable();
        dwm1001.leds.D12.enable();
        print!("Send...\n");

        // Start transmission
        dwm1001.DW1000
            .modify::<dw1000::SYS_CTRL, _>(|_, w|
                w
                    .txstrt(1)
            )
            .expect("Failed to modify register");

        // Wait until frame is sent
        loop {
            let sys_status = dwm1001.DW1000.read::<dw1000::SYS_STATUS>()
                .expect("Failed to read from register");

            if sys_status.txfrb() == 0b1 {
                dwm1001.DW1000.write::<dw1000::SYS_STATUS, _>(|w| w.txfrb(0b1))
                    .expect("Failed to reset flag");
                print!("Transmit Frame Begins\n");
            }
            if sys_status.txprs() == 0b1 {
                dwm1001.DW1000.write::<dw1000::SYS_STATUS, _>(|w| w.txprs(0b1))
                    .expect("Failed to reset flag");
                print!("Transmit Preamble Sent\n");
            }
            if sys_status.txphs() == 0b1 {
                dwm1001.DW1000.write::<dw1000::SYS_STATUS, _>(|w| w.txphs(0b1))
                    .expect("Failed to reset flag");
                print!("Transmit PHY Header Sent\n");
            }
            if sys_status.txfrs() == 0b1 {
                dwm1001.DW1000.write::<dw1000::SYS_STATUS, _>(|w| w.txfrs(0b1))
                    .expect("Failed to reset flag");
                print!("Transmit Frame Sent\n");
                break;
            }
        }

        dwm1001.leds.D12.disable();
    }
}
