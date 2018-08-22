//! Board support crate for the Decawave DWM1001/DWM1001-Dev
//!
//! This crate is in early development. Not much to see here, right now.


#![no_std]

#![deny(missing_docs)]
#![deny(warnings)]


pub extern crate cortex_m;
pub extern crate cortex_m_rt;
pub extern crate dw1000;
pub extern crate nrf52_hal;


use dw1000::DW1000;
use nrf52_hal::{
    prelude::*,
    gpio::{
        p0,
        Floating,
        Input,
    },
    nrf52::{
        self,
        CorePeripherals,
        Peripherals,
    },
    spim,
};

#[cfg(feature = "dev")]
use nrf52_hal::gpio::{
    p0::P0_Pin,
    Output,
    PushPull,
};


/// Provides access to all features of the DWM1001/DWM1001-Dev board
#[allow(non_snake_case)]
pub struct DWM1001 {
    /// The nRF52's pins
    pub pins: Pins,

    /// The LEDs on the DWM1001-Dev board
    ///
    /// This is only available if the `dev` feature is enabled.
    #[cfg(feature = "dev")]
    pub leds: Leds,

    /// DW1000 UWB transceiver
    pub DW1000: DW1000<nrf52::SPIM2>,

    /// Core peripheral: Cache and branch predictor maintenance operations
    pub CBP: nrf52::CBP,

    /// Core peripheral: CPUID
    pub CPUID: nrf52::CPUID,

    /// Core peripheral: Debug Control Block
    pub DCB: nrf52::DCB,

    /// Core peripheral: Data Watchpoint and Trace unit
    pub DWT: nrf52::DWT,

    /// Core peripheral: Flash Patch and Breakpoint unit
    pub FPB: nrf52::FPB,

    /// Core peripheral: Floating Point Unit
    pub FPU: nrf52::FPU,

    /// Core peripheral: Instrumentation Trace Macrocell
    pub ITM: nrf52::ITM,

    /// Core peripheral: Memory Protection Unit
    pub MPU: nrf52::MPU,

    /// Core peripheral: Nested Vector Interrupt Controller
    pub NVIC: nrf52::NVIC,

    /// Core peripheral: System Control Block
    pub SCB: nrf52::SCB,

    /// Core peripheral: SysTick Timer
    pub SYST: nrf52::SYST,

    /// Core peripheral: Trace Port Interface Unit
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

    /// nRF52 peripheral: TWIM1
    pub TWIM1: nrf52::TWIM1,

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
    /// Take the peripherals safely
    ///
    /// This method will return an instance of `DWM1001` the first time it is
    /// called. It will return only `None` on subsequent calls.
    pub fn take() -> Option<Self> {
        Some(Self::new(
            CorePeripherals::take()?,
            Peripherals::take()?,
        ))
    }

    /// Steal the peripherals
    ///
    /// This method produces an instance of `DWM1001`, regardless of whether
    /// another instance was create previously.
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
            sck : pins.p0_16.into_push_pull_output().degrade(),
            mosi: pins.p0_20.into_push_pull_output().degrade(),
            miso: pins.p0_18.into_floating_input().degrade(),
        });

        let dw_cs = pins.p0_17.into_push_pull_output().degrade();

        DWM1001 {
            pins: Pins {
                p0_02: pins.p0_2,
                p0_03: pins.p0_3,
                p0_04: pins.p0_4,
                p0_05: pins.p0_5,
                p0_06: pins.p0_6,
                p0_07: pins.p0_7,
                p0_08: pins.p0_8,
                p0_09: pins.p0_9,
                p0_10: pins.p0_10,
                p0_11: pins.p0_11,
                p0_12: pins.p0_12,
                p0_13: pins.p0_13,
                p0_15: pins.p0_15,
                p0_19: pins.p0_19,
                p0_21: pins.p0_21,
                p0_23: pins.p0_23,
                p0_24: pins.p0_24,
                p0_25: pins.p0_25,
                p0_26: pins.p0_26,
                p0_27: pins.p0_27,
                p0_28: pins.p0_28,
                p0_29: pins.p0_29,

                #[cfg(not(feature = "dev"))] p0_14: pins.p0_14,
                #[cfg(not(feature = "dev"))] p0_22: pins.p0_22,
                #[cfg(not(feature = "dev"))] p0_30: pins.p0_30,
                #[cfg(not(feature = "dev"))] p0_31: pins.p0_31,
            },

            #[cfg(feature = "dev")]
            leds: Leds {
                D9 : Led::new(pins.p0_30.degrade()),
                D10: Led::new(pins.p0_31.degrade()),
                D11: Led::new(pins.p0_22.degrade()),
                D12: Led::new(pins.p0_14.degrade()),
            },

            DW1000: DW1000::new(spim2, dw_cs),

            // Core peripherals
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
            TWIM1 : p.TWIM1,
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
/// The field names in this struct follow the naming convention of the nRF52.
/// Their documentation also states what the DWM1001 documentation calls them.
pub struct Pins {
    /// P0.02 - BT_WAKE_UP
    pub p0_02: p0::P0_2<Input<Floating>>,

    /// P0.03 - SPIS_CSn
    pub p0_03: p0::P0_3<Input<Floating>>,

    /// P0.04 - SPIS_CLK
    pub p0_04: p0::P0_4<Input<Floating>>,

    /// P0.05 - UART_TX
    pub p0_05: p0::P0_5<Input<Floating>>,

    /// P0.06 - SPIS_MOSI
    pub p0_06: p0::P0_6<Input<Floating>>,

    /// P0.07 - SPIS_MISO
    pub p0_07: p0::P0_7<Input<Floating>>,

    /// P0.08 - GPIO_8
    pub p0_08: p0::P0_8<Input<Floating>>,

    /// P0.09 - GPIO_9
    pub p0_09: p0::P0_9<Input<Floating>>,

    /// P0.10 - GPIO_10
    pub p0_10: p0::P0_10<Input<Floating>>,

    /// P0.11 - UART_RX
    pub p0_11: p0::P0_11<Input<Floating>>,

    /// P0.12 - GPIO_12
    pub p0_12: p0::P0_12<Input<Floating>>,

    /// P0.13 - GPIO_13
    pub p0_13: p0::P0_13<Input<Floating>>,

    /// P0.15 - GPIO_15
    pub p0_15: p0::P0_15<Input<Floating>>,

    /// P0.21 - RESETn
    pub p0_21: p0::P0_21<Input<Floating>>,

    /// P0.23 - GPIO_23
    pub p0_23: p0::P0_23<Input<Floating>>,

    /// P0.26 - READY
    pub p0_26: p0::P0_26<Input<Floating>>,

    /// P0.27 - GPIO_27
    pub p0_27: p0::P0_27<Input<Floating>>,

    /// P0.28 - I2C_SCL
    ///
    /// Connected to both the accelerometer and an outside pin.
    pub p0_28: p0::P0_28<Input<Floating>>,

    /// P0.29 - I2C_SDA
    ///
    /// Connected to both the accelerometer and an outside pin.
    pub p0_29: p0::P0_29<Input<Floating>>,

    /// P0.14 - GPIO_14
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for an LED on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub p0_14: p0::P0_14<Input<Floating>>,

    /// P0.22 - GPIO_22
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for an LED on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub p0_22: p0::P0_22<Input<Floating>>,

    /// P0.30 - GPIO_30
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for an LED on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub p0_30: p0::P0_30<Input<Floating>>,

    /// P0.31 - GPIO_31
    ///
    /// This field is only available, if the `dev` feature is disabled.
    /// Otherwise the pin is used for an LED on the DWM1001-Dev board.
    #[cfg(not(feature = "dev"))]
    pub p0_31: p0::P0_31<Input<Floating>>,

    // Pins before this comment are available outside the DWM1001. Pins after
    // this comment are connected to components on the board, and should
    // eventually be subsumed by higher-level abstractions.

    /// P0.19 - DW_IRQ
    ///
    /// Connected to the DW1000.
    pub p0_19: p0::P0_19<Input<Floating>>,

    /// P0.24 - DW_RST
    ///
    /// Connected to the DW1000.
    pub p0_24: p0::P0_24<Input<Floating>>,

    /// P0.25 - IRQ_ACC
    ///
    /// Connected to the accelerometer.
    pub p0_25: p0::P0_25<Input<Floating>>,
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
pub struct Led(p0::P0_Pin<Output<PushPull>>);

#[cfg(feature = "dev")]
impl Led {
    fn new<Mode>(pin: P0_Pin<Mode>) -> Self {
        let mut led = Led(pin.into_push_pull_output());
        led.disable();
        led
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
