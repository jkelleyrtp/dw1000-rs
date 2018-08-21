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


use nrf52_hal::nrf52::{
    self,
    CorePeripherals,
    Peripherals,
};


/// Provides access to all features of the DWM1001 board
#[allow(non_snake_case)]
pub struct DWM1001 {
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

    /// nRF52 peripheral: SPIM2
    pub SPIM2: nrf52::SPIM2,

    /// nRF52 peripheral: SPIS2
    pub SPIS2: nrf52::SPIS2,

    /// nRF52 peripheral: SPI2
    pub SPI2: nrf52::SPI2,

    /// nRF52 peripheral: RTC2
    pub RTC2: nrf52::RTC2,

    /// nRF52 peripheral: I2S
    pub I2S: nrf52::I2S,

    /// nRF52 peripheral: P0
    pub P0: nrf52::P0,
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
        DWM1001 {
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
            SPIM2 : p.SPIM2,
            SPIS2 : p.SPIS2,
            SPI2  : p.SPI2,
            RTC2  : p.RTC2,
            I2S   : p.I2S,
            P0    : p.P0,
        }
    }
}
