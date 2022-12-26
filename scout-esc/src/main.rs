#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = scout_st_spin_32::init();

    loop {
        Timer::after(Duration::from_millis(1)).await;
    }
}
