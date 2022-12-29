//! Models the STEVAL-SPIN3202 in the 6-step, sensorless (BEMF sensing) jumper configuration.

use embassy_stm32::gpio::{Input, Output, Pull};

use crate::sip;

/// This is a re-export from the SIP, for users of this BSP.
pub use sip::OverCurrentVisibility;

#[allow(non_snake_case)]
pub struct Peripherals {
    pub over_current_protection: OverCurrentProtection,
    pub OC_COMP_INT: Input<'static, embassy_stm32::peripherals::PB12>,
    pub OC_COMP_INT2: Input<'static, embassy_stm32::peripherals::PA12>,

    pub LS1: Output<'static, embassy_stm32::peripherals::PB13>,
    pub LS2: Output<'static, embassy_stm32::peripherals::PB14>,
    pub LS3: Output<'static, embassy_stm32::peripherals::PB15>,
    pub HS1: Output<'static, embassy_stm32::peripherals::PA8>,
    pub HS2: Output<'static, embassy_stm32::peripherals::PA9>,
    pub HS3: Output<'static, embassy_stm32::peripherals::PA10>,

    pub USER1: Input<'static, embassy_stm32::peripherals::PF0>,
    pub USER2: Input<'static, embassy_stm32::peripherals::PF1>,
    // TODO analog inputs
    // embassy-stm32 provides AdcPin trait, but this needs to be wrapped up in a struct
    // that also holds some kind of shared reference to the ADC (if these pins use the
    // same ADC peripheral)
    // pub R6: Input<'static, embassy_stm32::peripherals::PA3>,
    // pub BUS_VOLTAGE: Input<'static, embassy_stm32::peripherals::PB1>,
}

pub struct OverCurrentProtection {
    inner: sip::OverCurrentProtection,
}

pub enum OverCurrentThreshold {
    Standby,
    Amps20,
    Amps65,
    Amps140,
}

pub fn init() -> Peripherals {
    let p = sip::init();

    Peripherals {
        over_current_protection: OverCurrentProtection::init(p.over_current_protection),
        OC_COMP_INT: p.OC_COMP_INT,
        OC_COMP_INT2: p.OC_COMP_INT2,

        LS1: p.LS1,
        LS2: p.LS2,
        LS3: p.LS3,
        HS1: p.HS1,
        HS2: p.HS2,
        HS3: p.HS3,

        USER1: Input::new(p.PF0, Pull::None),
        USER2: Input::new(p.PF1, Pull::None),
    }
}

impl OverCurrentProtection {
    fn init(inner: sip::OverCurrentProtection) -> Self {
        Self { inner }
    }

    pub fn configure_visibility(&mut self, visibility: OverCurrentVisibility) {
        self.inner.configure_visibility(visibility)
    }

    pub fn configure_threshold(&mut self, threshold: OverCurrentThreshold) {
        let sip_threshold = match threshold {
            OverCurrentThreshold::Standby => sip::OverCurrentThreshold::Standby,
            OverCurrentThreshold::Amps20 => sip::OverCurrentThreshold::Millivolts100,
            OverCurrentThreshold::Amps65 => sip::OverCurrentThreshold::Millivolts250,
            OverCurrentThreshold::Amps140 => sip::OverCurrentThreshold::Millivolts500,
        };

        self.inner.configure_threshold(sip_threshold)
    }
}
