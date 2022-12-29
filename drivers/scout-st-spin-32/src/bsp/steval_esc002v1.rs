use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};

use crate::sip;

/// This is a re-export from the SIP, for users of this BSP.
pub use sip::OverCurrentVisibility;

#[allow(non_snake_case)]
pub struct Peripherals {
    pub PA5: embassy_stm32::peripherals::PA5,
    pub PA6: embassy_stm32::peripherals::PA6,
    pub PA7: embassy_stm32::peripherals::PA7,
    pub PA13: embassy_stm32::peripherals::PA13,
    pub PA14: embassy_stm32::peripherals::PA14,
    pub PA15: embassy_stm32::peripherals::PA15,
    pub PB6: embassy_stm32::peripherals::PB6,
    pub PB7: embassy_stm32::peripherals::PB7,

    pub over_current_protection: OverCurrentProtection,
    pub OC_COMP_INT: Input<'static, embassy_stm32::peripherals::PB12>,
    pub OC_COMP_INT2: Input<'static, embassy_stm32::peripherals::PA12>,

    pub BEMF_COMPARATOR_1: Input<'static, embassy_stm32::peripherals::PF1>,
    pub BEMF_COMPARATOR_2: Input<'static, embassy_stm32::peripherals::PF0>,
    pub BEMF_COMPARATOR_3: Input<'static, embassy_stm32::peripherals::PB1>,

    pub LS1: Output<'static, embassy_stm32::peripherals::PB13>,
    pub LS2: Output<'static, embassy_stm32::peripherals::PB14>,
    pub LS3: Output<'static, embassy_stm32::peripherals::PB15>,
    pub HS1: Output<'static, embassy_stm32::peripherals::PA8>,
    pub HS2: Output<'static, embassy_stm32::peripherals::PA9>,
    pub HS3: Output<'static, embassy_stm32::peripherals::PA10>,

    // LED control is inverted. Setting the output LOW turns on the LED.
    pub RGB_LED_RED: Output<'static, embassy_stm32::peripherals::PA0>,
    pub RGB_LED_GREEN: Output<'static, embassy_stm32::peripherals::PA1>,
    pub RGB_LED_BLUE: Output<'static, embassy_stm32::peripherals::PA2>,

    // Analog inputs
    pub ADC: embassy_stm32::peripherals::ADC,

    /// This input measured voltage on the power bus, scaled by a voltage divider. The
    /// actual voltage is measured voltage * 33 / 503.
    pub VBUS_FEEDBACK: embassy_stm32::peripherals::PA3,
    /// This input measures the voltage across the 0.001 ohm shunt resistor, amplified 25x. The
    /// actual voltage is measured voltage / 25. The actual current is actual voltage / 0.001.
    pub CURRENT_FEEDBACK: embassy_stm32::peripherals::PA4,
}

pub struct OverCurrentProtection {
    inner: sip::OverCurrentProtection,
}

pub enum OverCurrentThreshold {
    Standby,
    Amps100,
    Amps250,
    Amps500,
}

pub fn init() -> Peripherals {
    let p = sip::init();

    Peripherals {
        PA5: p.PA5,
        PA6: p.PA6,
        PA7: p.PA7,
        PA13: p.PA13,
        PA14: p.PA14,
        PA15: p.PA15,
        PB6: p.PB6,
        PB7: p.PB7,

        over_current_protection: OverCurrentProtection::init(p.over_current_protection),
        OC_COMP_INT: p.OC_COMP_INT,
        OC_COMP_INT2: p.OC_COMP_INT2,

        BEMF_COMPARATOR_1: Input::new(p.PF1, Pull::None),
        BEMF_COMPARATOR_2: Input::new(p.PF0, Pull::None),
        BEMF_COMPARATOR_3: Input::new(p.PB1, Pull::None),

        LS1: p.LS1,
        LS2: p.LS2,
        LS3: p.LS3,
        HS1: p.HS1,
        HS2: p.HS2,
        HS3: p.HS3,

        RGB_LED_RED: Output::new(p.PA0, Level::High, Speed::Medium),
        RGB_LED_GREEN: Output::new(p.PA1, Level::High, Speed::Medium),
        RGB_LED_BLUE: Output::new(p.PA2, Level::High, Speed::Medium),

        ADC: p.ADC,

        VBUS_FEEDBACK: p.PA3,
        CURRENT_FEEDBACK: p.PA4,
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
            OverCurrentThreshold::Amps100 => sip::OverCurrentThreshold::Millivolts100,
            OverCurrentThreshold::Amps250 => sip::OverCurrentThreshold::Millivolts250,
            OverCurrentThreshold::Amps500 => sip::OverCurrentThreshold::Millivolts500,
        };

        self.inner.configure_threshold(sip_threshold)
    }
}
