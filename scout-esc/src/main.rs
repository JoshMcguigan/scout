#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = embassy_stm32::init(Default::default());

    loop {
        Timer::after(Duration::from_millis(1)).await;
    }
}
