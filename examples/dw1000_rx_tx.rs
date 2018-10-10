//! Sends and receives data continuously
//!
//! This data sends and receives data continuously. To see it working in its
//! full glory, you need two DWM1001-DEV boards running it.
//!
//! As printing debug output via semihosting is really slow and would interefere
//! with the rest of the code, this example signals its status via LEDs. If
//! everything works well, you should see the blue LED blink from time to time
//! on both boards, signalling a successfully received message.


#![no_main]
#![no_std]

#![feature(nll)]


#[macro_use] extern crate cortex_m_rt;
#[macro_use] extern crate dwm1001;

extern crate heapless;
extern crate nb;
extern crate panic_semihosting;


use dwm1001::{
    prelude::*,
    debug,
    dw1000::{
        self,
        mac,
        DW1000,
    },
    nrf52832_hal::{
        Delay,
        Timer,
    },
    DWM1001,
};
use heapless::FnvIndexSet;


#[entry]
fn main() -> ! {
    debug::init();

    let mut known_nodes = FnvIndexSet::<_, heapless::consts::U64>::new();

    let mut dwm1001 = DWM1001::take().unwrap();

    let     clocks = dwm1001.CLOCK.constrain().freeze();
    let mut delay  = Delay::new(dwm1001.SYST, clocks);
    let mut rng    = dwm1001.RNG.constrain();

    dwm1001.DW_RST.reset_dw1000(&mut delay);
    let mut dw1000 = dwm1001.DW1000.init()
        .expect("Failed to initialize DW1000");

    // Set network address
    let address = rng.random_u16();
    dw1000
        .set_address(
            mac::Address {
                pan_id:     0x0d57,  // hardcoded network id
                short_addr: address, // random device address
            }
        )
        .expect("Failed to set address");

    // Configure timer
    let mut task_timer    = dwm1001.TIMER0.constrain();
    let mut timeout_timer = dwm1001.TIMER1.constrain();
    let mut output_timer  = dwm1001.TIMER2.constrain();

    let receive_time = 500_000 + (rng.random_u32() % 500_000);

    output_timer.start(5_000_000);

    loop {
        task_timer.start(receive_time);
        loop {
            match task_timer.wait() {
                Ok(()) =>
                    break,
                Err(nb::Error::WouldBlock) =>
                    (),
                Err(_) =>
                    unreachable!(),
            }

            timeout_timer.start(100_000);
            match receive(&mut dw1000, &mut timeout_timer) {
                Ok(source) => {
                    // Sucessfully received: Blue LED
                    dwm1001.leds.D10.enable();
                    delay.delay_ms(30u32);
                    dwm1001.leds.D10.disable();

                    if let Err(_) = known_nodes.insert(source) {
                        print!("Too many nodes. Can't add another one.\n");
                    }
                }
                Err(_) => {
                    // It would be nice to print the error, but that takes way
                    // too much time and interferes with everything else.
                    continue;
                }
            }
        }

        task_timer.start(50_000);
        loop {
            match task_timer.wait() {
                Ok(()) =>
                    break,
                Err(nb::Error::WouldBlock) =>
                    (),
                Err(_) =>
                    unreachable!(),
            }

            timeout_timer.start(10_000);
            match send(&mut dw1000, &mut timeout_timer) {
                Ok(()) => {
                    ()
                }
                Err(_) => {
                    // It would be nice to print the error, but that takes way
                    // too much time and interferes with everything else.
                    continue;
                }
            }
        }

        if output_timer.wait().is_ok() {
            print!("\n-- Known nodes:\n");
            for node in &known_nodes {
                print!("PAN ID: 0x{:04x}, Short Address: 0x{:04x}\n",
                    node.pan_id,
                    node.short_addr,
                );
            }

            output_timer.start(5_000_000);
        }
    }
}

fn receive<SPI, T>(
    dw1000: &mut DW1000<SPI, dw1000::Ready>,
    timer:  &mut Timer<T>,
)
    -> Result<mac::Address, Error>
    where
        SPI: SpimExt,
        T:   TimerExt,
{
    let mut buffer = [0u8; 1024];

    let mut future = dw1000.receive()?;

    // Wait until frame has been received
    let frame = loop {
        match future.wait(&mut buffer) {
            Ok(frame) =>
                break frame,
            Err(nb::Error::WouldBlock) =>
                (),
            Err(nb::Error::Other(error)) =>
                return Err(Error::Dw1000(error)),
        }

        match timer.wait() {
            Ok(()) =>
                return Err(Error::Timeout),
            Err(nb::Error::WouldBlock) =>
                (),
            Err(_) =>
                unreachable!(),
        }
    };

    if frame.payload != b"ping" {
        return Err(Error::UnexpectedMessage);
    }

    Ok(frame.header.source)
}

fn send<SPI, T>(
    dw1000: &mut DW1000<SPI, dw1000::Ready>,
    timer:  &mut Timer<T>,
)
    -> Result<(), Error>
    where
        SPI: SpimExt,
        T:   TimerExt,
{
    let mut future = dw1000.send(b"ping", mac::Address::broadcast(), None)?;

    loop {
        match future.wait() {
            Ok(()) =>
                break,
            Err(nb::Error::WouldBlock) =>
                (),
            Err(nb::Error::Other(error)) =>
                return Err(Error::Dw1000(error)),
        }

        match timer.wait() {
            Ok(()) =>
                return Err(Error::Timeout),
            Err(nb::Error::WouldBlock) =>
                (),
            Err(_) =>
                unreachable!(),
        }
    }

    Ok(())
}


#[derive(Debug)]
pub enum Error {
    Dw1000(dw1000::Error),
    Timeout,
    UnexpectedMessage,
}

impl From<dw1000::Error> for Error {
    fn from(error: dw1000::Error) -> Self {
        Error::Dw1000(error)
    }
}
