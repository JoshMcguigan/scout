#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use panic_probe as _;

use cortex_m::peripheral::NVIC;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{AnyPin, Input, Level, Output},
    interrupt,
    pac::Interrupt,
    peripherals::TIM1,
    pwm::{simple_pwm::SimplePwm, Channel},
    timer::low_level::AdvancedControlInstance,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};

use scout_st_spin_32::{
    bsp::steval_esc002v1::{OverCurrentThreshold, Peripherals},
    sip::OverCurrentVisibility,
};

/// Channel used on TIM1 to trigger a read on the BEMF comparator.
const BEMF_READ_WAKEUP_TIMER_CHANNEL: usize = 3;

const STARTUP_PER_PHASE_DURATION: Duration = Duration::from_micros(2000);

static BEMF_SIGNAL: Signal<CriticalSectionRawMutex, Level> = Signal::new();
static mut ACTIVE_BEMF_INPUT: Option<&'static Input<AnyPin>> = None;
static mut BEMF_COMPARATOR_1: Option<Input<AnyPin>> = None;
static mut BEMF_COMPARATOR_2: Option<Input<AnyPin>> = None;
static mut BEMF_COMPARATOR_3: Option<Input<AnyPin>> = None;

enum MotorState {
    Starting,
    Running,
}

#[interrupt]
fn TIM1_CC() {
    // Safety: This value is populated before this interrupt is enabled.
    let input = unsafe { ACTIVE_BEMF_INPUT.unwrap_unchecked() };

    BEMF_SIGNAL.signal(input.get_level());

    // Clear the timer interrupt.
    unsafe {
        TIM1::regs_advanced().sr().write(|w| {
            w.0 = u32::MAX;
            w.set_ccif(BEMF_READ_WAKEUP_TIMER_CHANNEL, false);
        });
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let Peripherals {
        mut over_current_protection,
        mut hs,
        ls1,
        ls2,
        ls3,
        BEMF_COMPARATOR_1: bemf_comparator_1,
        BEMF_COMPARATOR_2: bemf_comparator_2,
        BEMF_COMPARATOR_3: bemf_comparator_3,
        ..
    } = scout_st_spin_32::bsp::steval_esc002v1::init();

    let mut ls1 = ls1.degrade();
    let mut ls2 = ls2.degrade();
    let mut ls3 = ls3.degrade();

    // Safety: These are only read from an interrupt which isn't enabled yet, so
    // they are safe to write to.
    unsafe {
        BEMF_COMPARATOR_1 = Some(bemf_comparator_1.degrade());
        BEMF_COMPARATOR_2 = Some(bemf_comparator_2.degrade());
        BEMF_COMPARATOR_3 = Some(bemf_comparator_3.degrade());
    }

    over_current_protection.configure_threshold(OverCurrentThreshold::Amps500);
    over_current_protection.configure_visibility(OverCurrentVisibility::McuOnly);

    let duty = (hs.get_max_duty() as f32 * 0.1) as u16;
    let bemf_check_time = duty / 2;
    hs.set_duty(Channel::Ch1, duty);
    hs.set_duty(Channel::Ch2, duty);
    hs.set_duty(Channel::Ch3, duty);

    unsafe {
        // Globally unmask the interrupts for this timer. This is not the same as enabling
        // the interrupts, which we do not do here.
        NVIC::unmask(Interrupt::TIM1_CC);

        // Generally we'd need to setup the ARR register here, but that is already
        // done since we are using this same timer for PWM.

        TIM1::regs_advanced()
            .ccr(BEMF_READ_WAKEUP_TIMER_CHANNEL)
            .modify(|r| r.set_ccr(bemf_check_time));
        TIM1::regs_advanced().cr1().modify(|r| r.set_cen(true));

        // The timer is setup and running, but we haven't enabled the interrupts yet.
    };

    let mut state = MotorState::Starting;

    loop {
        // phase 1
        switch_high_side(&mut hs, Channel::Ch2, Channel::Ch1, unsafe {
            &BEMF_COMPARATOR_2
        });
        match state {
            MotorState::Starting => Timer::after(STARTUP_PER_PHASE_DURATION).await,
            MotorState::Running => wait_for_bemf(Level::Low).await,
        }

        // phase 2
        switch_low_side(&mut ls3, &mut ls2, unsafe { &BEMF_COMPARATOR_3 });
        match state {
            MotorState::Starting => Timer::after(STARTUP_PER_PHASE_DURATION).await,
            MotorState::Running => wait_for_bemf(Level::High).await,
        }

        // phase 3
        switch_high_side(&mut hs, Channel::Ch1, Channel::Ch3, unsafe {
            &BEMF_COMPARATOR_1
        });
        match state {
            MotorState::Starting => Timer::after(STARTUP_PER_PHASE_DURATION).await,
            MotorState::Running => wait_for_bemf(Level::Low).await,
        }

        // phase 4
        switch_low_side(&mut ls2, &mut ls1, unsafe { &BEMF_COMPARATOR_2 });
        match state {
            MotorState::Starting => Timer::after(STARTUP_PER_PHASE_DURATION).await,
            MotorState::Running => wait_for_bemf(Level::High).await,
        }

        // phase 5
        switch_high_side(&mut hs, Channel::Ch3, Channel::Ch2, unsafe {
            &BEMF_COMPARATOR_3
        });
        match state {
            MotorState::Starting => Timer::after(STARTUP_PER_PHASE_DURATION).await,
            MotorState::Running => wait_for_bemf(Level::Low).await,
        }

        // phase 6
        switch_low_side(&mut ls1, &mut ls3, unsafe { &BEMF_COMPARATOR_1 });
        match state {
            MotorState::Starting => Timer::after(STARTUP_PER_PHASE_DURATION).await,
            MotorState::Running => wait_for_bemf(Level::High).await,
        }

        state = MotorState::Running;
    }
}

async fn wait_for_bemf(expected_bemf_level: Level) {
    loop {
        BEMF_SIGNAL.reset();
        let bemf_comparator_level = BEMF_SIGNAL.wait().await;
        if bemf_comparator_level == expected_bemf_level {
            break;
        }
    }
}

fn switch_high_side(
    hs: &mut SimplePwm<'static, TIM1>,
    from: Channel,
    to: Channel,
    active_comparator: &'static Option<Input<'static, AnyPin>>,
) {
    set_bemf_read_wakeup_interrupt_state(false);

    hs.disable(from);
    hs.enable(to);

    unsafe {
        ACTIVE_BEMF_INPUT = Some(active_comparator.as_ref().unwrap_unchecked());
    }

    set_bemf_read_wakeup_interrupt_state(true);
}

fn switch_low_side(
    from: &mut Output<'static, AnyPin>,
    to: &mut Output<'static, AnyPin>,
    active_comparator: &'static Option<Input<'static, AnyPin>>,
) {
    set_bemf_read_wakeup_interrupt_state(false);

    from.set_low();
    to.set_high();

    unsafe {
        ACTIVE_BEMF_INPUT = Some(active_comparator.as_ref().unwrap_unchecked());
    }

    set_bemf_read_wakeup_interrupt_state(true);
}

fn set_bemf_read_wakeup_interrupt_state(interrupt_enabled: bool) {
    unsafe {
        TIM1::regs_advanced()
            .dier()
            .modify(|r| r.set_ccie(BEMF_READ_WAKEUP_TIMER_CHANNEL, interrupt_enabled));
    }
}
