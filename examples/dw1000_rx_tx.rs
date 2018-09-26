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


#[macro_use] extern crate cortex_m_rt;

extern crate cortex_m_semihosting;
extern crate dwm1001;
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
        nrf52::RNG,
        Delay,
        Timer,
    },
    DWM1001,
};


#[entry]
fn main() -> ! {
    debug::init();

    let mut dwm1001 = DWM1001::take().unwrap();

    let     clocks = dwm1001.CLOCK.constrain().freeze();
    let mut delay  = Delay::new(dwm1001.SYST, clocks);

    dwm1001.DW_RST.reset_dw1000(&mut delay);

    // Set network address
    let address = random_u16(&mut dwm1001.RNG);
    dwm1001.DW1000
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

    let receive_time = 500_000 + (random_u32(&mut dwm1001.RNG) % 500_000);

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
            match receive(&mut dwm1001.DW1000, &mut timeout_timer) {
                Ok(()) => {
                    // Sucessfully received: Blue LED
                    dwm1001.leds.D10.enable();
                    delay.delay_ms(30u32);
                    dwm1001.leds.D10.disable();
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
            match send(&mut dwm1001.DW1000, &mut timeout_timer) {
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
    }
}

fn random_u16(rng: &mut RNG) -> u16 {
    let mut val = 0u16;

    rng.tasks_start.write(|w| unsafe { w.bits(1) });
    for i in 0 ..= 1 {
        while rng.events_valrdy.read().bits() == 0 {}
        rng.events_valrdy.write(|w| unsafe { w.bits(0) });

        val |= (rng.value.read().value().bits() as u16) << (i * 8);
    }

    val
}

fn random_u32(rng: &mut RNG) -> u32 {
    let mut val = 0u32;

    rng.tasks_start.write(|w| unsafe { w.bits(1) });
    for i in 0 ..= 3 {
        while rng.events_valrdy.read().bits() == 0 {}
        rng.events_valrdy.write(|w| unsafe { w.bits(0) });

        val |= (rng.value.read().value().bits() as u32) << (i * 8);
    }

    val
}

fn receive<SPI, T>(
    dw1000: &mut DW1000<SPI>,
    timer:  &mut Timer<T>,
)
    -> Result<(), Error>
    where
        SPI: SpimExt,
        T:   TimerExt,
{
    let mut buffer = [0u8; 1024];

    let mut future = dw1000.receive()?;

    // Wait until frame has been received
    let len = loop {
        match future.wait(&mut buffer) {
            Ok(len) =>
                break len,
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

    if len < 2 {
        return Err(Error::UnexpectedMessage);
    }
    if !buffer[.. len-2].ends_with(b"ping") {
        return Err(Error::UnexpectedMessage);
    }

    Ok(())
}

fn send<SPI, T>(
    dw1000: &mut DW1000<SPI>,
    timer:  &mut Timer<T>,
)
    -> Result<(), Error>
    where
        SPI: SpimExt,
        T:   TimerExt,
{
    let mut future = dw1000.send(b"ping")?;

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
