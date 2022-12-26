#![no_std]

mod nrf24_syma;
pub use nrf24_syma::SymaX5C;

mod nrf24l01;

#[derive(defmt::Format)]
pub struct FourChannelRadioData {
    pub throttle: u8,
    pub yaw: i8,
    pub pitch: i8,
    pub roll: i8,
}
