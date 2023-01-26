#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_stm32::{peripherals::TIM1, pwm::Channel, timer::low_level::AdvancedControlInstance};
use embassy_time::{Duration, Instant};

use scout_st_spin_32::{
    bsp::steval_esc002v1::{OverCurrentThreshold, Peripherals},
    sip::OverCurrentVisibility,
};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let Peripherals {
        mut over_current_protection,
        mut hs,
        mut ls1,
        mut ls2,
        mut ls3,
        BEMF_COMPARATOR_1,
        BEMF_COMPARATOR_2,
        BEMF_COMPARATOR_3,
        ..
    } = scout_st_spin_32::bsp::steval_esc002v1::init();

    over_current_protection.configure_threshold(OverCurrentThreshold::Amps500);
    over_current_protection.configure_visibility(OverCurrentVisibility::McuOnly);

    let duty = (hs.get_max_duty() as f32 * 0.1) as u16;
    let check_range_min = duty / 2;
    let check_range_max = 3 * duty / 4;
    hs.set_duty(Channel::Ch1, duty);
    hs.set_duty(Channel::Ch2, duty);
    hs.set_duty(Channel::Ch3, duty);

    let startup_per_phase_duration = Duration::from_micros(2000);
    let mut steady_state = false;

    loop {
        // phase 1
        hs.disable(Channel::Ch2);
        hs.enable(Channel::Ch1);

        let start = Instant::now();
        if steady_state {
            let mut i = 0;
            loop {
                // Safety: Reading from this register does not have any side effects.
                let pwm_count = unsafe { TIM1::regs_advanced().cnt().read() }.cnt();
                // The polling for this could be replaced by using channel 4 of TIM1 to setup
                // an interrupt triggering this read, but for now this seems to perform well enough.
                if check_range_min < pwm_count && pwm_count < check_range_max {
                    if BEMF_COMPARATOR_2.is_low() {
                        i += 1;
                    } else {
                        i = 0;
                    }
                }

                if i > 3 {
                    break;
                }

                // TODO handle a missing pulse using some timeout?
            }
        } else {
            loop {
                if start.elapsed() > startup_per_phase_duration {
                    break;
                }
            }
        }

        // phase 2
        ls3.set_low();
        ls2.set_high();
        let start = Instant::now();
        if steady_state {
            let mut i = 0;
            loop {
                // Safety: Reading from this register does not have any side effects.
                let pwm_count = unsafe { TIM1::regs_advanced().cnt().read() }.cnt();
                // The polling for this could be replaced by using channel 4 of TIM1 to setup
                // an interrupt triggering this read, but for now this seems to perform well enough.
                if check_range_min < pwm_count && pwm_count < check_range_max {
                    if BEMF_COMPARATOR_3.is_high() {
                        i += 1;
                    } else {
                        i = 0;
                    }
                }

                if i > 3 {
                    break;
                }

                // TODO handle a missing pulse using some timeout?
            }
        } else {
            loop {
                if start.elapsed() > startup_per_phase_duration {
                    break;
                }
            }
        }

        // phase 3
        hs.disable(Channel::Ch1);
        hs.enable(Channel::Ch3);
        let start = Instant::now();
        if steady_state {
            let mut i = 0;
            loop {
                // Safety: Reading from this register does not have any side effects.
                let pwm_count = unsafe { TIM1::regs_advanced().cnt().read() }.cnt();
                // The polling for this could be replaced by using channel 4 of TIM1 to setup
                // an interrupt triggering this read, but for now this seems to perform well enough.
                if check_range_min < pwm_count && pwm_count < check_range_max {
                    if BEMF_COMPARATOR_1.is_low() {
                        i += 1;
                    } else {
                        i = 0;
                    }
                }

                if i > 3 {
                    break;
                }

                // TODO handle a missing pulse using some timeout?
            }
        } else {
            loop {
                if start.elapsed() > startup_per_phase_duration {
                    break;
                }
            }
        }

        // phase 4
        ls2.set_low();
        ls1.set_high();
        let start = Instant::now();
        if steady_state {
            let mut i = 0;
            loop {
                // Safety: Reading from this register does not have any side effects.
                let pwm_count = unsafe { TIM1::regs_advanced().cnt().read() }.cnt();
                // The polling for this could be replaced by using channel 4 of TIM1 to setup
                // an interrupt triggering this read, but for now this seems to perform well enough.
                if check_range_min < pwm_count && pwm_count < check_range_max {
                    if BEMF_COMPARATOR_2.is_high() {
                        i += 1;
                    } else {
                        i = 0;
                    }
                }

                if i > 3 {
                    break;
                }

                // TODO handle a missing pulse using some timeout?
            }
        } else {
            loop {
                if start.elapsed() > startup_per_phase_duration {
                    break;
                }
            }
        }

        // phase 5
        hs.disable(Channel::Ch3);
        hs.enable(Channel::Ch2);
        let start = Instant::now();
        if steady_state {
            let mut i = 0;
            loop {
                // Safety: Reading from this register does not have any side effects.
                let pwm_count = unsafe { TIM1::regs_advanced().cnt().read() }.cnt();
                // The polling for this could be replaced by using channel 4 of TIM1 to setup
                // an interrupt triggering this read, but for now this seems to perform well enough.
                if check_range_min < pwm_count && pwm_count < check_range_max {
                    if BEMF_COMPARATOR_3.is_low() {
                        i += 1;
                    } else {
                        i = 0;
                    }
                }

                if i > 3 {
                    break;
                }

                // TODO handle a missing pulse using some timeout?
            }
        } else {
            loop {
                if start.elapsed() > startup_per_phase_duration {
                    break;
                }
            }
        }

        // phase 6
        ls1.set_low();
        ls3.set_high();
        let start = Instant::now();
        if steady_state {
            let mut i = 0;
            loop {
                // Safety: Reading from this register does not have any side effects.
                let pwm_count = unsafe { TIM1::regs_advanced().cnt().read() }.cnt();
                // The polling for this could be replaced by using channel 4 of TIM1 to setup
                // an interrupt triggering this read, but for now this seems to perform well enough.
                if check_range_min < pwm_count && pwm_count < check_range_max {
                    if BEMF_COMPARATOR_1.is_high() {
                        i += 1;
                    } else {
                        i = 0;
                    }
                }

                if i > 3 {
                    break;
                }

                // TODO handle a missing pulse using some timeout?
            }
        } else {
            loop {
                if start.elapsed() > startup_per_phase_duration {
                    break;
                }
            }
        }

        steady_state = true;
    }
}
