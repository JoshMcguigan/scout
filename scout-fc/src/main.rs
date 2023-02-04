#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{error, println, unwrap};
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    i2c::{self, I2c},
    interrupt::{self, Interrupt},
    peripherals::{DMA2_CH0, DMA2_CH3, SPI1},
    spi::{self, Spi},
    time::{mhz, Hertz},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Delay, Duration, Instant, Timer};
use static_cell::StaticCell;

use panic_probe as _;

use scout_nrf24l01::SymaX5C;

type SpiBus1 = embassy_stm32::spi::Spi<'static, SPI1, DMA2_CH3, DMA2_CH0>;
static SPI_BUS: StaticCell<Mutex<ThreadModeRawMutex, SpiBus1>> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    // PB8 is labeled D15 on the NUCLEO-F446RE
    // PB9 is labeled D14 on the NUCLEO-F446RE
    // See UM1724 Table 19
    let scl = p.PB8;
    let sda = p.PB9;
    let i2c_config = {
        let mut c = i2c::Config::default();
        c.sda_pullup = true;
        c.scl_pullup = true;
        c
    };
    let mut tail_rotor_esc_i2c = I2c::new(
        p.I2C1,
        scl,
        sda,
        // Safety: This interrupt isn't used anywhere else.
        //
        // TODO why do I have to steal this? Should it be part of the Peripheral singleton?
        unsafe { interrupt::I2C1_EV::steal() },
        p.DMA1_CH0,
        p.DMA1_CH1,
        Hertz::khz(2),
        i2c_config,
    );

    let spi_bus_1 = {
        // PA5 is labeled D13 on the NUCLEO-F446RE
        // PA6 is labeled D12 on the NUCLEO-F446RE
        // PA7 is labeled D11 on the NUCLEO-F446RE
        // See UM1724 Table 19
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
    // PA8 is labeled D7 on the NUCLEO-F446RE
    // See UM1724 Table 19
    let csn = Output::new(p.PA8, Level::High, Speed::High);

    let spi_dev_1 = SpiDevice::new(spi_bus_1, csn);

    // PA9 is labeled D8 on the NUCLEO-F446RE
    // See UM1724 Table 19
    let ce = Output::new(p.PA9, Level::Low, Speed::High);

    let mut radio = unwrap!(SymaX5C::new(spi_dev_1, ce, Delay).await);

    loop {
        match radio.read().await {
            Ok(Some(radio_data)) => {
                println!("{:?}", radio_data);
                let i2c_write_start = Instant::now();
                let i2c_write_timeout = Duration::from_millis(1000);
                let tail_rotor_write_result =
                    tail_rotor_esc_i2c.blocking_write_timeout(47, &[234], || {
                        if i2c_write_start.elapsed() > i2c_write_timeout {
                            Err(i2c::Error::Timeout)
                        } else {
                            Ok(())
                        }
                    });
                println!("{:?}", tail_rotor_write_result);
            }
            Ok(None) => {
                // No new radio packet available
            }
            Err(e) => error!("{:?}", e),
        }

        Timer::after(Duration::from_millis(1)).await;
    }
}
