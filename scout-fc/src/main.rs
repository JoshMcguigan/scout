#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{error, println, unwrap};
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    peripherals::{DMA2_CH0, DMA2_CH3, SPI1},
    spi::{self, Spi},
    time::mhz,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Timer};
use static_cell::StaticCell;

use panic_probe as _;

use scout_nrf24l01::SymaX5C;

type SpiBus1 = embassy_stm32::spi::Spi<'static, SPI1, DMA2_CH3, DMA2_CH0>;
static SPI_BUS: StaticCell<Mutex<ThreadModeRawMutex, SpiBus1>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let spi_bus_1 = {
        let sck = p.PA5;
        let miso = p.PA6;
        let mosi = p.PA7;
        let spi = Spi::new(
            p.SPI1,
            sck,
            mosi,
            miso,
            p.DMA2_CH3,
            p.DMA2_CH0,
            mhz(1),
            spi::Config::default(),
        );
        SPI_BUS.init(Mutex::<ThreadModeRawMutex, _>::new(spi))
    };
    let csn = Output::new(p.PB6, Level::High, Speed::High);
    let spi_dev_1 = SpiDevice::new(spi_bus_1, csn);

    // PC7 is labeled D9 on the NUCLEO-F446RE
    // See UM1724 Table 19
    let ce = Output::new(p.PC7, Level::Low, Speed::High);

    let mut radio = unwrap!(SymaX5C::new(spi_dev_1, ce, Delay).await);

    loop {
        match radio.read().await {
            Ok(Some(radio_data)) => {
                println!("{:?}", radio_data);
            }
            Ok(None) => {
                // No new radio packet available
            }
            Err(e) => error!("{:?}", e),
        }

        Timer::after(Duration::from_millis(1)).await;
    }
}
