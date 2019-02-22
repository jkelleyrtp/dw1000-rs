//! Board support crate for the Decawave DWM1001
//!
//! Provides access to the features of the [DWM1001 Module] and the
//! [DWM1001 Development Board]. Most notably, this includes the Decawave
//! [DW1000 Radio IC] and the Nordic [nRF52832] microcontroller.
//!
//! The entry point to the API is the [DWM1001] struct. You can find example
//! programs using the API in the [examples directory].
//!
//! [DWM1001 Module]: https://www.decawave.com/product/dwm1001-module/
//! [DWM1001 Development Board]: https://www.decawave.com/product/dwm1001-development-board/
//! [DW1000 Radio IC]: https://www.decawave.com/product/dw1000-radio-ic/
//! [nRF52832]: https://www.nordicsemi.com/Products/Low-power-short-range-wireless/nRF52832
//! [examples directory]: https://github.com/braun-robotics/rust-dwm1001/tree/master/examples


#![no_std]

#![deny(missing_docs)]


pub use cortex_m;
pub use cortex_m_rt;
pub use dw1000;
pub use embedded_hal;

pub use nrf52832_hal;

pub use dw1000::{
    block_timeout,
    repeat_timeout,
};

/// Exports traits that are usually needed when using this crate
pub mod prelude {
    pub use nrf52832_hal::prelude::*;
}

pub mod debug;


use cortex_m::{
    asm,
    interrupt,
};
use dw1000::DW1000;
use embedded_hal::blocking::delay::DelayMs;
use nrf52832_hal::{
    prelude::*,
    gpio::{
        p0,
        Floating,
        Input,
        Level,
        OpenDrainConfig,
        Output,
        PushPull,
    },
    nrf52832_pac::{
        self as nrf52,
        CorePeripherals,
        Interrupt,
        Peripherals,
    },
    spim,
    twim,
    Timer,
    Spim,
    Twim,
};

#[cfg(feature = "dev")]
use nrf52832_hal::{
    gpio::Pin,
    uarte::{
        self,
        Uarte,
        Parity as UartParity,
        Baudrate as UartBaudrate,
    },
};


/// Provides access to the features of the DWM1001/DWM1001-Dev board
///
/// You can get an instance of this struct by using [`DWM1001::take`] or
/// [`DWM1001::steal`].
#[allow(non_snake_case)]
pub struct DWM1001 {
    /// The nRF52's pins
    pub pins: Pins,

    /// The LEDs on the DWM1001-Dev board
    ///
    /// This is only available if the `dev` feature is enabled.
    #[cfg(feature = "dev")]
    pub leds: Leds,

    /// DWM1001 UART, wired to USB virtual UART port
    ///
    /// This is only available if the `dev` feature is enabled.
    #[cfg(feature = "dev")]
    pub uart: Uarte<nrf52::UARTE0>,

    /// The DW_RST pin (P0.24 on the nRF52)
    ///
    /// Can be used to reset the DW1000 externally.
    pub DW_RST: DW_RST,

    /// The DW_IRQ pin (P0.19 on the nRF52)
    ///
    /// Can be used to wait for DW1000 interrupts.
    pub DW_IRQ: DW_IRQ,

    /// The Decawave DW1000 Radio IC
    pub DW1000: DW1000<
        Spim<nrf52::SPIM2>,
        p0::P0_17<Output<PushPull>>,
        dw1000::Uninitialized
    >,

    /// LIS2DH12 3-axis accelerometer
    ///
    /// So far no Rust driver exists for the LIS2DH12, so this provides direct
    /// access to the I2C bus it's connected to.
    pub LIS2DH12: Twim<nrf52::TWIM1>,

    /// nRF52 nRF52 core peripheral: Cache and branch predictor maintenance
    /// operations
    pub CBP: nrf52::CBP,

    /// nRF52 core peripheral: CPUID
    pub CPUID: nrf52::CPUID,

    /// nRF52 core peripheral: Debug Control Block
    pub DCB: nrf52::DCB,

    /// nRF52 core peripheral: Data Watchpoint and Trace unit
    pub DWT: nrf52::DWT,

    /// nRF52 core peripheral: Flash Patch and Breakpoint unit
    pub FPB: nrf52::FPB,

    /// nRF52 core peripheral: Floating Point Unit
    pub FPU: nrf52::FPU,

    /// nRF52 core peripheral: Instrumentation Trace Macrocell
    pub ITM: nrf52::ITM,

    /// nRF52 core peripheral: Memory Protection Unit
    pub MPU: nrf52::MPU,

    /// nRF52 core peripheral: Nested Vector Interrupt Controller
    pub NVIC: nrf52::NVIC,

    /// nRF52 core peripheral: System Control Block
    pub SCB: nrf52::SCB,

    /// nRF52 core peripheral: SysTick Timer
    pub SYST: nrf52::SYST,

    /// nRF52 core peripheral: Trace Port Interface Unit
    pub TPIU: nrf52::TPIU,

    /// nRF52 peripheral: FICR
    pub FICR: nrf52::FICR,

    /// nRF52 peripheral: UICR
    pub UICR: nrf52::UICR,

    /// nRF52 peripheral: BPROT
    pub BPROT: nrf52::BPROT,

    /// nRF52 peripheral: POWER
    pub POWER: nrf52::POWER,

    /// nRF52 peripheral: CLOCK
    pub CLOCK: nrf52::CLOCK,

    /// nRF52 peripheral: RADIO
    pub RADIO: nrf52::RADIO,

    /// nRF52 peripheral: UARTE0
    #[cfg(not(feature = "dev"))]
    pub UARTE0: nrf52::UARTE0,

    /// nRF52 peripheral: UART0
    pub UART0: nrf52::UART0,

    /// nRF52 peripheral: SPIM0
    pub SPIM0: nrf52::SPIM0,

    /// nRF52 peripheral: SPIS0
    pub SPIS0: nrf52::SPIS0,

    /// nRF52 peripheral: TWIM0
    pub TWIM0: nrf52::TWIM0,

    /// nRF52 peripheral: TWIS0
    pub TWIS0: nrf52::TWIS0,

    /// nRF52 peripheral: SPI0
    pub SPI0: nrf52::SPI0,

    /// nRF52 peripheral: TWI0
    pub TWI0: nrf52::TWI0,

    /// nRF52 peripheral: SPIM1
    pub SPIM1: nrf52::SPIM1,

    /// nRF52 peripheral: SPIS1
    pub SPIS1: nrf52::SPIS1,

    /// nRF52 peripheral: TWIS1
    pub TWIS1: nrf52::TWIS1,

    /// nRF52 peripheral: SPI1
    pub SPI1: nrf52::SPI1,

    /// nRF52 peripheral: TWI1
    pub TWI1: nrf52::TWI1,

    /// nRF52 peripheral: NFCT
    pub NFCT: nrf52::NFCT,

    /// nRF52 peripheral: GPIOTE
    pub GPIOTE: nrf52::GPIOTE,

    /// nRF52 peripheral: SAADC
    pub SAADC: nrf52::SAADC,

    /// nRF52 peripheral: TIMER0
    pub TIMER0: nrf52::TIMER0,

    /// nRF52 peripheral: TIMER1
    pub TIMER1: nrf52::TIMER1,

    /// nRF52 peripheral: TIMER2
    pub TIMER2: nrf52::TIMER2,

    /// nRF52 peripheral: RTC0
    pub RTC0: nrf52::RTC0,

    /// nRF52 peripheral: TEMP
    pub TEMP: nrf52::TEMP,

    /// nRF52 peripheral: RNG
    pub RNG: nrf52::RNG,

    /// nRF52 peripheral: ECB
    pub ECB: nrf52::ECB,

    /// nRF52 peripheral: CCM
    pub CCM: nrf52::CCM,

    /// nRF52 peripheral: AAR
    pub AAR: nrf52::AAR,

    /// nRF52 peripheral: WDT
    pub WDT: nrf52::WDT,

    /// nRF52 peripheral: RTC1
    pub RTC1: nrf52::RTC1,

    /// nRF52 peripheral: QDEC
    pub QDEC: nrf52::QDEC,

    /// nRF52 peripheral: COMP
    pub COMP: nrf52::COMP,

    /// nRF52 peripheral: LPCOMP
    pub LPCOMP: nrf52::LPCOMP,

    /// nRF52 peripheral: SWI0
    pub SWI0: nrf52::SWI0,

    /// nRF52 peripheral: EGU0
    pub EGU0: nrf52::EGU0,

    /// nRF52 peripheral: SWI1
    pub SWI1: nrf52::SWI1,

    /// nRF52 peripheral: EGU1
    pub EGU1: nrf52::EGU1,

    /// nRF52 peripheral: SWI2
    pub SWI2: nrf52::SWI2,

    /// nRF52 peripheral: EGU2
    pub EGU2: nrf52::EGU2,

    /// nRF52 peripheral: SWI3
    pub SWI3: nrf52::SWI3,

    /// nRF52 peripheral: EGU3
    pub EGU3: nrf52::EGU3,

    /// nRF52 peripheral: SWI4
    pub SWI4: nrf52::SWI4,

    /// nRF52 peripheral: EGU4
    pub EGU4: nrf52::EGU4,

    /// nRF52 peripheral: SWI5
    pub SWI5: nrf52::SWI5,

    /// nRF52 peripheral: EGU5
    pub EGU5: nrf52::EGU5,

    /// nRF52 peripheral: TIMER3
    pub TIMER3: nrf52::TIMER3,

    /// nRF52 peripheral: TIMER4
    pub TIMER4: nrf52::TIMER4,

    /// nRF52 peripheral: PWM0
    pub PWM0: nrf52::PWM0,

    /// nRF52 peripheral: PDM
    pub PDM: nrf52::PDM,

    /// nRF52 peripheral: NVMC
    pub NVMC: nrf52::NVMC,

    /// nRF52 peripheral: PPI
    pub PPI: nrf52::PPI,

    /// nRF52 peripheral: MWU
    pub MWU: nrf52::MWU,

    /// nRF52 peripheral: PWM1
    pub PWM1: nrf52::PWM1,

    /// nRF52 peripheral: PWM2
    pub PWM2: nrf52::PWM2,

    /// nRF52 peripheral: RTC2
    pub RTC2: nrf52::RTC2,

    /// nRF52 peripheral: I2S
    pub I2S: nrf52::I2S,
}

impl DWM1001 {
    /// Take ownership of a `DWM1001` instance safely
    ///
    /// To uphold a numer of safety guarantees made by this crate's UI, only one
    /// instance of `DWM1001` must exist at any given time.
    ///
    /// This method will return an instance of `DWM1001` the first time it is
    /// called. It will return only `None` on subsequent calls.
    pub fn take() -> Option<Self> {
        Some(Self::new(
            CorePeripherals::take()?,
            Peripherals::take()?,
        ))
    }

    /// Take ownership of a `DWM1001` instance, circumventing safety guarantees
    ///
    /// This method produces an instance of `DWM1001`, regardless of whether
    /// another instance was created previously.
    ///
    /// # Safety
    ///
    /// This method can be used to create multiple instances of `DWM1001`. Those
    /// instances can interfere with each other, causing all kinds of unexpected
    /// behavior and circumventing safety guarantees in many ways.
    ///
    /// Always use `DWM1001::take`, unless you really know what you're doing.
    pub unsafe fn steal() -> Self {
        Self::new(
            CorePeripherals::steal(),
            Peripherals::steal(),
        )
    }

    fn new(cp: CorePeripherals, p: Peripherals) -> Self {
        let pins = p.P0.split();

        // Some notes about the hardcoded configuration of `Spim`:
        // - The DW1000's SPI mode can be configured, but on the DWM1001 board,
        //   both configuration pins (GPIO5/SPIPOL and GPIO6/SPIPHA) are
        //   unconnected and internally pulled low, setting it to SPI mode 0.
        // - The frequency is set to a moderate value that the DW1000 can easily
        //   handle.
        let spim2 = p.SPIM2.constrain(spim::Pins {
            sck : pins.p0_16.into_push_pull_output(Level::Low).degrade(),
            mosi: Some(pins.p0_20.into_push_pull_output(Level::Low).degrade()),
            miso: Some(pins.p0_18.into_floating_input().degrade()),
        });

        let twim1 = p.TWIM1.constrain(
            twim::Pins {
                scl: pins.p0_28.into_floating_input().degrade(),
                sda: pins.p0_29.into_floating_input().degrade(),
            },
            twim::Frequency::K250,
        );

        let dw_cs = pins.p0_17.into_push_pull_output(Level::High);

        // Some notes about the hardcoded configuration of `Uarte`:
        // - On the DWM1001-DEV board, the UART is connected (without CTS/RTS
        //   flow control) to the attached debugger chip. This UART is exposed
        //   via USB as a virtual port, which is capable of 1Mbps baudrate (but
        //   not reliably!).
        // - Although these ports/pins are exposed generally on the DWM1001
        //   package, and are marked as UART RXD and TXD, they are not
        //   necessarily used as such by the firmware. For this reason,
        //   non-`dev` features may be used to manually configure the serial
        //   port.
        #[cfg(feature = "dev")]
        let uarte0 = p.UARTE0.constrain(uarte::Pins {
                txd: pins.p0_05.into_push_pull_output(Level::High).degrade(),
                rxd: pins.p0_11.into_floating_input().degrade(),
                cts: None,
                rts: None,
            },
            UartParity::EXCLUDED,
            UartBaudrate::BAUD460800
        );

        DWM1001 {
            #[cfg(feature = "dev")]
            uart: uarte0,

            pins: Pins {
                BT_WAKE_UP: pins.p0_02,
                SPIS_CSn  : pins.p0_03,
                SPIS_CLK  : pins.p0_04,
                SPIS_MOSI : pins.p0_06,
                SPIS_MISO : pins.p0_07,
                RESETn    : pins.p0_21,
                READY     : pins.p0_26,

                GPIO_8 : pins.p0_08,
                GPIO_9 : pins.p0_09,
                GPIO_10: pins.p0_10,
                GPIO_12: pins.p0_12,
                GPIO_13: pins.p0_13,
                GPIO_15: pins.p0_15,
                GPIO_23: pins.p0_23,
                GPIO_27: pins.p0_27,

                #[cfg(not(feature = "dev"))] UART_RX   : pins.p0_11,
                #[cfg(not(feature = "dev"))] UART_TX   : pins.p0_05,

                #[cfg(not(feature = "dev"))] GPIO_14: pins.p0_14,
                #[cfg(not(feature = "dev"))] GPIO_22: pins.p0_22,
                #[cfg(not(feature = "dev"))] GPIO_30: pins.p0_30,
                #[cfg(not(feature = "dev"))] GPIO_31: pins.p0_31,

                IRQ_ACC: pins.p0_25,
            },

            #[cfg(feature = "dev")]
            leds: Leds {
                D9 : Led::new(pins.p0_30.degrade()),
                D10: Led::new(pins.p0_31.degrade()),
                D11: Led::new(pins.p0_22.degrade()),
                D12: Led::new(pins.p0_14.degrade()),
            },

            DW_RST: DW_RST::new(pins.p0_24),
            DW_IRQ: DW_IRQ::new(pins.p0_19),

            DW1000: DW1000::new(spim2, dw_cs),

            LIS2DH12: twim1,

            // nRF52 core peripherals
            CBP  : cp.CBP,
            CPUID: cp.CPUID,
            DCB  : cp.DCB,
            DWT  : cp.DWT,
            FPB  : cp.FPB,
            FPU  : cp.FPU,
            ITM  : cp.ITM,
            MPU  : cp.MPU,
            NVIC : cp.NVIC,
            SCB  : cp.SCB,
            SYST : cp.SYST,
            TPIU : cp.TPIU,

            // nRF52 peripherals
            FICR  : p.FICR,
            UICR  : p.UICR,
            BPROT : p.BPROT,
            POWER : p.POWER,
            CLOCK : p.CLOCK,
            RADIO : p.RADIO,

            #[cfg(not(feature = "dev"))]
            UARTE0: p.UARTE0,

            UART0 : p.UART0,
            SPIM0 : p.SPIM0,
            SPIS0 : p.SPIS0,
            TWIM0 : p.TWIM0,
            TWIS0 : p.TWIS0,
            SPI0  : p.SPI0,
            TWI0  : p.TWI0,
            SPIM1 : p.SPIM1,
            SPIS1 : p.SPIS1,
            TWIS1 : p.TWIS1,
            SPI1  : p.SPI1,
            TWI1  : p.TWI1,
            NFCT  : p.NFCT,
            GPIOTE: p.GPIOTE,
            SAADC : p.SAADC,
            TIMER0: p.TIMER0,
            TIMER1: p.TIMER1,
            TIMER2: p.TIMER2,
            RTC0  : p.RTC0,
            TEMP  : p.TEMP,
            RNG   : p.RNG,
            ECB   : p.ECB,
            CCM   : p.CCM,
            AAR   : p.AAR,
            WDT   : p.WDT,
            RTC1  : p.RTC1,
            QDEC  : p.QDEC,
            COMP  : p.COMP,
            LPCOMP: p.LPCOMP,
            SWI0  : p.SWI0,
            EGU0  : p.EGU0,
            SWI1  : p.SWI1,
            EGU1  : p.EGU1,
            SWI2  : p.SWI2,
            EGU2  : p.EGU2,
            SWI3  : p.SWI3,
            EGU3  : p.EGU3,
            SWI4  : p.SWI4,
            EGU4  : p.EGU4,
            SWI5  : p.SWI5,
            EGU5  : p.EGU5,
            TIMER3: p.TIMER3,
            TIMER4: p.TIMER4,
            PWM0  : p.PWM0,
            PDM   : p.PDM,
            NVMC  : p.NVMC,
            PPI   : p.PPI,
            MWU   : p.MWU,
            PWM1  : p.PWM1,
            PWM2  : p.PWM2,
            RTC2  : p.RTC2,
            I2S   : p.I2S,
        }
    }
}


/// The nRF52 pins that are available on the DWM1001
///
/// The documentation of the fields states the names of the pin on the DWM1001
/// and the nRF52.
#[allow(non_snake_case)]
pub struct Pins {
    /// DWM1001: BT_WAKE_UP; nRF52: P0.02
    pub BT_WAKE_UP: p0::P0_02<Input<Floating>>,

    /// DWM1001: SPIS_CSn; nRF52: P0.03
    pub SPIS_CSn: p0::P0_03<Input<Floating>>,

    /// DWM1001: SPIS_CLK; nRF52: P0.04
    pub SPIS_CLK: p0::P0_04<Input<Floating>>,

    /// DWM1001: UART_TX; nRF52: P0.05
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for a UART on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub UART_TX: p0::P0_05<Input<Floating>>,

    /// DWM1001: SPIS_MOSI; nRF52: P0.06
    pub SPIS_MOSI: p0::P0_06<Input<Floating>>,

    /// DWM1001: SPIS_MISO; nRF52: P0.07
    pub SPIS_MISO: p0::P0_07<Input<Floating>>,

    /// DWM1001: UART_RX; nRF52: P0.11
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for a UART on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub UART_RX: p0::P0_11<Input<Floating>>,

    /// DWM1001: RESETn; nRF52: P0.21
    pub RESETn: p0::P0_21<Input<Floating>>,

    /// DWM1001: READY; nRF52: P0.26
    pub READY: p0::P0_26<Input<Floating>>,

    /// DWM1001: GPIO_8; nRF52: P0.08
    pub GPIO_8: p0::P0_08<Input<Floating>>,

    /// DWM1001: GPIO_9; nRF52: P0.09
    pub GPIO_9: p0::P0_09<Input<Floating>>,

    /// DWM1001: GPIO_10; nRF52: P0.10
    pub GPIO_10: p0::P0_10<Input<Floating>>,

    /// DWM1001: GPIO_12; nRF52: P0.12
    pub GPIO_12: p0::P0_12<Input<Floating>>,

    /// DWM1001: GPIO_13; nRF52: P0.13
    pub GPIO_13: p0::P0_13<Input<Floating>>,

    /// DWM1001: GPIO_15; nRF52: P0.15
    pub GPIO_15: p0::P0_15<Input<Floating>>,

    /// DWM1001: GPIO_23; nRF52: P0.23
    pub GPIO_23: p0::P0_23<Input<Floating>>,

    /// DWM1001: GPIO_27; nRF52: P0.27
    pub GPIO_27: p0::P0_27<Input<Floating>>,

    /// DWM1001: GPIO_14; nRF52: P0.14
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for an LED on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub GPIO_14: p0::P0_14<Input<Floating>>,

    /// DWM1001: GPIO_22; nRF52: P0.22
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for an LED on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub GPIO_22: p0::P0_22<Input<Floating>>,

    /// DWM1001: GPIO_30; nRF52: P0.30
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for an LED on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub GPIO_30: p0::P0_30<Input<Floating>>,

    /// DWM1001: GPIO_31; nRF52: P0.31
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for an LED on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub GPIO_31: p0::P0_31<Input<Floating>>,

    // Pins before this comment are available outside the DWM1001. Pins after
    // this comment are connected to components on the board, and should
    // eventually be subsumed by higher-level abstractions.

    /// DWM1001: IRQ_ACC; nRF52: P0.25
    ///
    /// Connected to the accelerometer.
    pub IRQ_ACC: p0::P0_25<Input<Floating>>,
}


/// The LEDs on the DWM1001-Dev board
///
/// The documentation of the field's states the name of the LED on the
/// DWM1001-Dev, as well as the names of the pins on the DWM1001 and nRF52.
///
/// This struct is only available, if the `dev` feature is enabled.
#[allow(non_snake_case)]
#[cfg(feature = "dev")]
pub struct Leds {
    /// DWM1001-Dev: D9; DWM1001: GPIO_30; nRF52: P0.30
    pub D9: Led,

    /// DWM1001-Dev: D10; DWM1001: GPIO_31; nRF52: P0.31
    pub D10: Led,

    /// DWM1001-Dev: D11; DWM1001: GPIO_22; nRF52: P0.22
    pub D11: Led,

    /// DWM1001-Dev: D12; DWM1001: GPIO_14; nRF52: P0.14
    pub D12: Led,
}


/// An LED on the DWM1001-Dev board
///
/// This struct is only available, if the `dev` feature is enabled.
#[cfg(feature = "dev")]
pub struct Led(Pin<Output<PushPull>>);

#[cfg(feature = "dev")]
impl Led {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Led(pin.into_push_pull_output(Level::High))
    }

    /// Enable the LED
    pub fn enable(&mut self) {
        self.0.set_low()
    }

    /// Disable the LED
    pub fn disable(&mut self) {
        self.0.set_high()
    }
}


/// The DW_RST pin (P0.24 on the nRF52)
///
/// Can be used to externally reset the DW1000.
#[allow(non_camel_case_types)]
pub struct DW_RST(Option<p0::P0_24<Input<Floating>>>);

impl DW_RST {
    fn new<Mode>(p0_24: p0::P0_24<Mode>) -> Self {
        DW_RST(Some(p0_24.into_floating_input()))
    }

    /// Externally reset the DW1000 using its RSTn pin
    ///
    /// The implementation of this method needs to wait a few times until the
    /// DW1000 is properly reset. To do that, it requires an implementation of
    /// [`DelayMs`] from the `embedded-hal` crate, which the user must supply.
    ///
    /// See [`nrf52832_hal::Delay`] for such an implementation.
    pub fn reset_dw1000<D>(&mut self, delay: &mut D) where D: DelayMs<u32> {
        // This whole `Option` thing is a bit of a hack. What we actually need
        // here is the ability to put the pin into a tri-state mode that allows
        // us to switch input/output on the fly.
        let dw_rst = self.0
            .take()
            .unwrap()
            // According the the DW1000 datasheet (section 5.6.3.1), the reset
            // pin should be pulled low using open-drain, and must never be
            // pulled high.
            .into_open_drain_output(
                OpenDrainConfig::Standard0Disconnect1,
                Level::Low
            );

        // Section 5.6.3.1 in the data sheet talks about keeping this low for
        // T-RST_OK, which would be 10-50 nanos. But table 15 makes it sound
        // like that should actually be T-DIG_ON (1.5-2 millis), which lines up
        // with the example code I looked at.
        delay.delay_ms(2);

        self.0 = Some(dw_rst.into_floating_input());

        // There must be some better way to determine whether the DW1000 is
        // ready, but I guess waiting for some time will do.
        delay.delay_ms(5);
    }
}


/// The DW_IRQ pin (P0.19 on the nRF52)
///
/// Can be used to wait for DW1000 interrupts.
#[allow(non_camel_case_types)]
pub struct DW_IRQ(p0::P0_19<Input<Floating>>);

impl DW_IRQ {
    fn new<Mode>(p0_19: p0::P0_19<Mode>) -> Self {
        DW_IRQ(p0_19.into_floating_input())
    }

    /// Sets up DW1000 interrupt and goes to sleep until an interrupt occurs
    ///
    /// This method sets up the interrupt of the pin connected to DW_IRQ on the
    /// DW1000 and goes to sleep, waiting for interrupts.
    ///
    /// There are two gotchas that must be kept in mind when using this method:
    /// - This method returns on _any_ interrupt, even those unrelated to the
    ///   DW1000.
    /// - This method disables interrupt handlers. No interrupt handler will be
    ///   called while this method is active.
    pub fn wait_for_interrupts<T>(&mut self,
        nvic:   &mut nrf52::NVIC,
        gpiote: &mut nrf52::GPIOTE,
        timer:  &mut Timer<T>,
    )
        where T: TimerExt
    {
        gpiote.config[0].write(|w| {
            let w = w
                .mode().event()
                .polarity().lo_to_hi();

            unsafe { w.psel().bits(19) }
        });
        gpiote.intenset.modify(|_, w| w.in0().set());

        interrupt::free(|_| {
            nrf52::NVIC::unpend(Interrupt::GPIOTE);
            nrf52::NVIC::unpend(T::INTERRUPT);

            nvic.enable(Interrupt::GPIOTE);
            timer.enable_interrupt(nvic);

            asm::dsb();
            asm::wfi();

            // If we don't do this, the (probably non-existing) interrupt
            // handler will be called as soon as we exit this closure.
            nvic.disable(Interrupt::GPIOTE);
            timer.disable_interrupt(nvic);
        });

        gpiote.events_in[0].write(|w| unsafe { w.bits(0) });
        gpiote.intenclr.modify(|_, w| w.in0().clear());
    }
}
