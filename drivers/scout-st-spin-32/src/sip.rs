use embassy_stm32::{
    gpio::{AnyPin, Input, Level, Output, Pin, Pull, Speed},
    pwm::simple_pwm::{PwmPin, SimplePwm},
    time::Hertz,
    Config,
};

#[allow(non_snake_case)]
pub struct Peripherals {
    pub PF0: embassy_stm32::peripherals::PF0,
    pub PF1: embassy_stm32::peripherals::PF1,
    pub PA0: embassy_stm32::peripherals::PA0,
    pub PA1: embassy_stm32::peripherals::PA1,
    pub PA2: embassy_stm32::peripherals::PA2,
    pub PA3: embassy_stm32::peripherals::PA3,
    pub PA4: embassy_stm32::peripherals::PA4,
    pub PA5: embassy_stm32::peripherals::PA5,
    pub PA6: embassy_stm32::peripherals::PA6,
    pub PA7: embassy_stm32::peripherals::PA7,
    pub PA13: embassy_stm32::peripherals::PA13,
    pub PA14: embassy_stm32::peripherals::PA14,
    pub PA15: embassy_stm32::peripherals::PA15,
    pub PB1: embassy_stm32::peripherals::PB1,
    pub PB6: embassy_stm32::peripherals::PB6,
    pub PB7: embassy_stm32::peripherals::PB7,

    pub EXTI0: embassy_stm32::peripherals::EXTI0,
    pub EXTI1: embassy_stm32::peripherals::EXTI1,

    pub over_current_protection: OverCurrentProtection,
    pub OC_COMP_INT: Input<'static, embassy_stm32::peripherals::PB12>,
    pub OC_COMP_INT2: Input<'static, embassy_stm32::peripherals::PA12>,

    /// High side outputs
    pub hs: SimplePwm<'static, embassy_stm32::peripherals::TIM1>,
    /// Low side output 1
    pub ls1: Output<'static, AnyPin>,
    /// Low side output 2
    pub ls2: Output<'static, AnyPin>,
    /// Low side output 3
    pub ls3: Output<'static, AnyPin>,

    pub ADC: embassy_stm32::peripherals::ADC,
}

pub struct OverCurrentProtection {
    oc_sel: Output<'static, embassy_stm32::peripherals::PA11>,
    oc_th_stdby_1: Output<'static, embassy_stm32::peripherals::PF7>,
    oc_th_stdby_2: Output<'static, embassy_stm32::peripherals::PF6>,
}

pub enum OverCurrentVisibility {
    /// OC comparator output signal is visible only to MCU (default)
    McuOnly,
    /// OC comparator output signal is visible to MCU and also acts
    /// on gate driver control logic
    McuAndOutputDriver,
}

pub enum OverCurrentThreshold {
    Standby,
    Millivolts100,
    Millivolts250,
    Millivolts500,
}

pub fn init() -> Peripherals {
    let config = {
        let mut c = Config::default();
        c.rcc.sys_ck = Some(Hertz::mhz(48));
        c
    };
    let p = embassy_stm32::init(config);

    let hs1 = PwmPin::new_ch1(p.PA8);
    let hs2 = PwmPin::new_ch2(p.PA9);
    let hs3 = PwmPin::new_ch3(p.PA10);
    let ls1 = Output::new(p.PB13.degrade(), Level::Low, Speed::VeryHigh);
    let ls2 = Output::new(p.PB14.degrade(), Level::Low, Speed::VeryHigh);
    let ls3 = Output::new(p.PB15.degrade(), Level::Low, Speed::VeryHigh);

    Peripherals {
        PF0: p.PF0,
        PF1: p.PF1,
        PA0: p.PA0,
        PA1: p.PA1,
        PA2: p.PA2,
        PA3: p.PA3,
        PA4: p.PA4,
        PA5: p.PA5,
        PA6: p.PA6,
        PA7: p.PA7,
        PA13: p.PA13,
        PA14: p.PA14,
        PA15: p.PA15,
        PB1: p.PB1,
        PB6: p.PB6,
        PB7: p.PB7,

        EXTI0: p.EXTI0,
        EXTI1: p.EXTI1,

        over_current_protection: OverCurrentProtection::init(p.PA11, p.PF6, p.PF7),
        OC_COMP_INT: Input::new(p.PB12, Pull::None),
        OC_COMP_INT2: Input::new(p.PA12, Pull::None),

        hs: SimplePwm::new(
            p.TIM1,
            Some(hs1),
            Some(hs2),
            Some(hs3),
            None,
            Hertz::khz(10),
        ),
        ls1,
        ls2,
        ls3,

        ADC: p.ADC,
    }
}

impl OverCurrentProtection {
    /// Initializes the over-current protection circuit in standby mode.
    fn init(
        pa11: embassy_stm32::peripherals::PA11,
        pf6: embassy_stm32::peripherals::PF6,
        pf7: embassy_stm32::peripherals::PF7,
    ) -> Self {
        Self {
            oc_sel: Output::new(pa11, Level::Low, Speed::Low),
            oc_th_stdby_1: Output::new(pf7, Level::Low, Speed::Low),
            oc_th_stdby_2: Output::new(pf6, Level::Low, Speed::Low),
        }
    }

    pub fn configure_visibility(&mut self, visibility: OverCurrentVisibility) {
        let level = match visibility {
            OverCurrentVisibility::McuOnly => Level::Low,
            OverCurrentVisibility::McuAndOutputDriver => Level::High,
        };

        self.oc_sel.set_level(level);
    }

    pub fn configure_threshold(&mut self, threshold: OverCurrentThreshold) {
        let (th1, th2) = match threshold {
            OverCurrentThreshold::Standby => (Level::Low, Level::Low),
            OverCurrentThreshold::Millivolts100 => (Level::High, Level::Low),
            OverCurrentThreshold::Millivolts250 => (Level::Low, Level::High),
            OverCurrentThreshold::Millivolts500 => (Level::High, Level::High),
        };

        self.oc_th_stdby_1.set_level(th1);
        self.oc_th_stdby_2.set_level(th2);
    }
}
