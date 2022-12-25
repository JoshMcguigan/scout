//! Driver for the NRF24L01
//!
//! This is not intended to be used by application code directly. Rather
//! it should be wrapped by modules implementing the various wire protocols.

use embedded_hal::{
    digital::{self, OutputPin},
    spi,
};
use embedded_hal_async::spi::{transaction, SpiBus, SpiBusRead, SpiBusWrite, SpiDevice};

pub struct Nrf23L01Plus<SPI, CE> {
    spi: SPI,
    chip_enable: CE,
}

impl<SPI, CE> Nrf23L01Plus<SPI, CE>
where
    SPI: SpiDevice,
    CE: OutputPin,
    SPI::Bus: SpiBus<u8>,
{
    pub async fn new(
        spi: SPI,
        mut chip_enable: CE,
    ) -> Result<
        Self,
        TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>,
    > {
        chip_enable.set_low().map_err(TransferError::Pin)?;

        Ok(Self { spi, chip_enable })
    }

    pub async fn set_chip_enable(
        &mut self,
        chip_enable: bool,
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        self.chip_enable
            .set_state(chip_enable.into())
            .map_err(TransferError::Pin)
    }

    pub async fn configure(
        &mut self,
        config_update: config_register_write::ConfigRegisterWrite,
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        let current_config = self.read_register(Register::CONFIG).await?;

        let new_config = config_update.apply_on_top_of(current_config);

        self.write_register(Register::CONFIG, new_config).await?;

        Ok(())
    }

    pub async fn set_auto_ack(
        &mut self,
        auto_ack_setting: bool,
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        self.write_register(
            Register::EN_AA,
            if auto_ack_setting {
                0b0011_1111
            } else {
                0b0000_0000
            },
        )
        .await?;

        Ok(())
    }

    // TODO this currently disables all other channels
    pub async fn enable_rx_addr(
        &mut self,
        channel: u8,
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        assert!(channel <= 5);

        self.write_register(Register::EN_RXADDR, 1u8 << channel)
            .await?;

        Ok(())
    }

    pub async fn rf_setup(
        &mut self,
        // TODO this should take structured data similar to configure
        desired_register_value: u8,
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        self.write_register(Register::RF_SETUP, desired_register_value)
            .await?;

        Ok(())
    }

    pub async fn set_rx_addr_p0(
        &mut self,
        rx_addr: &[u8],
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        assert!(rx_addr.len() <= 5);
        self.write_register_multi(Register::RX_ADDR_P0, rx_addr)
            .await?;

        Ok(())
    }

    pub async fn set_channel(
        &mut self,
        channel: u8,
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        self.write_register(Register::RF_CH, channel).await?;

        Ok(())
    }

    pub async fn set_payload_size_p0(
        &mut self,
        payload_size: u8,
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        self.write_register(Register::RX_PW_P0, payload_size)
            .await?;

        Ok(())
    }

    pub async fn read(
        &mut self,
        buf: &mut [u8],
    ) -> Result<
        bool,
        TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>,
    > {
        if (self.read_register(Register::FIFO_STATUS).await? & 0b0000_0001) == 0 {
            // RX queue not empty, so we read from it

            transaction!(&mut self.spi, move |bus| async move {
                bus.write(&[Instruction::read_rx_payload().as_byte()])
                    .await?;

                bus.read(buf).await?;

                Ok(())
            })
            .await
            .map_err(TransferError::Spi)?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn read_register(
        &mut self,
        register: Register,
    ) -> Result<u8, TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        // Both bytes are written on the SPI bus, but the radio
        // ignores the second byte and uses those clock pulses to
        // send a response.
        let mut buf = [Instruction::read_register(register).as_byte(), 0];

        self.spi
            .transfer_in_place(&mut buf)
            .await
            .map_err(TransferError::Spi)?;

        Ok(buf[1])
    }

    // This isn't implemented as a wrapper around write_register_multi because it is slightly
    // faster to only call the SPI write function once.
    async fn write_register(
        &mut self,
        register: Register,
        value: u8,
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        let buf = [Instruction::write_register(register).as_byte(), value];

        self.spi.write(&buf).await.map_err(TransferError::Spi)?;

        Ok(())
    }

    async fn write_register_multi(
        &mut self,
        register: Register,
        values: &[u8],
    ) -> Result<(), TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>>
    {
        transaction!(&mut self.spi, move |bus| async move {
            bus.write(&[Instruction::write_register(register).as_byte()])
                .await?;

            bus.write(values).await?;
            Ok(())
        })
        .await
        .map_err(TransferError::Spi)?;

        Ok(())
    }
}

#[derive(defmt::Format)]
pub enum TransferError<SPIError, PinError> {
    Spi(SPIError),
    Pin(PinError),
}

pub mod config_register_write {
    #[allow(dead_code)]
    pub enum Mode {
        /// Mnemonic PRX
        Rx,
        /// Mnemonic PTX
        Tx,
    }

    #[allow(dead_code)]
    pub enum Power {
        On,
        Off,
    }

    #[allow(dead_code)]
    pub enum Crc {
        Disabled,
        OneByte,
        TwoBytes,
    }

    /// Defines a write to the CONFIG register. `Option::None` values
    /// are not written.
    #[derive(Default)]
    pub struct ConfigRegisterWrite {
        /// Mnemonic PRIM_RX
        pub mode: Option<Mode>,
        /// Mnemonic PWR_UP
        pub power: Option<Power>,
        /// Mnemonic EN_CRC/CRC0
        pub crc: Option<Crc>,
    }

    impl ConfigRegisterWrite {
        pub(super) fn apply_on_top_of(self, mut existing_config: u8) -> u8 {
            if let Some(mode) = self.mode {
                existing_config = match mode {
                    Mode::Rx => existing_config | 0b0000_0001,
                    Mode::Tx => existing_config & 0b1111_1110,
                };
            }
            if let Some(power) = self.power {
                existing_config = match power {
                    Power::On => existing_config | 0b0000_0010,
                    Power::Off => existing_config & 0b1111_1101,
                };
            }
            if let Some(crc) = self.crc {
                existing_config = match crc {
                    Crc::Disabled => existing_config & 0b1111_0111,
                    Crc::OneByte => existing_config | 0b0000_1000 & 0b1111_1011,
                    Crc::TwoBytes => existing_config | 0b0000_1100,
                };
            }

            existing_config
        }
    }
}

#[allow(dead_code, non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Clone, Copy)]
pub(crate) enum Register {
    CONFIG = 0x0,
    EN_AA = 0x1,
    EN_RXADDR = 0x2,
    SETUP_AW = 0x3,
    SETUP_RETR = 0x4,
    RF_CH = 0x5,
    RF_SETUP = 0x6,
    STATUS = 0x7,
    OBSERVE_TX = 0x8,
    CD = 0x9,
    RX_ADDR_P0 = 0xa,
    RX_ADDR_P1 = 0xb,
    RX_ADDR_P2 = 0xc,
    RX_ADDR_P3 = 0xd,
    RX_ADDR_P4 = 0xe,
    RX_ADDR_P5 = 0xf,
    TX_ADDR = 0x10,
    RX_PW_P0 = 0x11,
    RX_PW_P1 = 0x12,
    RX_PW_P2 = 0x13,
    RX_PW_P3 = 0x14,
    RX_PW_P4 = 0x15,
    RX_PW_P5 = 0x16,
    FIFO_STATUS = 0x17,
    DYNPD = 0x1c,
    FEATURE = 0x1d,
}

impl Register {
    pub(crate) fn addr(&self) -> u8 {
        *self as u8
    }
}

struct Instruction(u8);

impl Instruction {
    fn as_byte(&self) -> u8 {
        self.0
    }

    fn read_register(register: Register) -> Self {
        let read_register_command = 0b0000_0000;

        Self(read_register_command | register.addr())
    }

    fn read_rx_payload() -> Self {
        Self(0b0110_0001)
    }

    fn write_register(register: Register) -> Self {
        let write_register_command = 0b0010_0000;

        Self(write_register_command | register.addr())
    }
}
