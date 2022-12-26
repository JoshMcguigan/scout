//! Implementation of Syma protocols on top of the NRF24L01 radio

use embedded_hal::{
    digital::{self, OutputPin},
    spi,
};
use embedded_hal_async::{
    delay::DelayUs,
    spi::{SpiBus, SpiDevice},
};

use super::{
    nrf24l01::{
        config_register_write::{self, ConfigRegisterWrite},
        Nrf23L01Plus, TransferError,
    },
    FourChannelRadioData,
};

const PAYLOAD_SIZE: usize = 16;

const NUM_DATA_CHANNELS: usize = 15;
const DATA_CHANNELS: [u8; NUM_DATA_CHANNELS] = [
    0x1d, 0x2f, 0x26, 0x3d, 0x15, 0x2b, 0x25, 0x24, 0x27, 0x2c, 0x1c, 0x3e, 0x39, 0x2d, 0x22,
];
const ADDR_LEN: usize = 5;
const ADDR: [u8; ADDR_LEN] = [0x6d, 0x6a, 0x73, 0x73, 0x73];

pub struct SymaX5C<SPI, CE> {
    radio: Nrf23L01Plus<SPI, CE>,
    latest_packet: [u8; PAYLOAD_SIZE],
    current_channel_idx: usize,
    packets_received_on_current_channel: usize,
}

impl<SPI, CE> SymaX5C<SPI, CE>
where
    SPI: SpiDevice,
    CE: OutputPin,
    SPI::Bus: SpiBus<u8>,
{
    pub async fn new<DELAY: DelayUs>(
        spi: SPI,
        chip_enable: CE,
        mut delay: DELAY,
    ) -> Result<
        Self,
        TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>,
    > {
        // TODO The driver setup should include this delay
        // TODO this error should be bubbled up in the result type
        //      but that requires a new SetupError result type
        // Data sheet specifies 100ms power on reset time
        delay.delay_ms(100).await.unwrap();

        let mut radio = Nrf23L01Plus::new(spi, chip_enable).await?;

        radio
            .configure(ConfigRegisterWrite {
                mode: Some(config_register_write::Mode::Rx),
                power: Some(config_register_write::Power::On),
                crc: Some(config_register_write::Crc::TwoBytes),
            })
            .await?;

        // TODO The driver setup should include this delay
        // TODO this error should be bubbled up in the result type
        // Data sheet specifies 1.5ms delay time after setting PWR_ON
        delay.delay_ms(1500).await.unwrap();

        radio.set_auto_ack(false).await?;
        radio.enable_rx_addr(0).await?;
        // 1mbps and -12dBm
        // TODO the driver should expose a nicer interface here
        radio.rf_setup(0b0000_0010).await?;

        radio.set_rx_addr_p0(&ADDR).await?;
        let current_channel_idx = 0;
        radio
            .set_channel(DATA_CHANNELS[current_channel_idx])
            .await?;
        radio.set_payload_size_p0(PAYLOAD_SIZE as u8).await?;
        radio.set_chip_enable(true).await?;

        Ok(Self {
            radio,
            latest_packet: [0; PAYLOAD_SIZE],
            current_channel_idx,
            packets_received_on_current_channel: 0,
        })
    }

    pub async fn read(
        &mut self,
    ) -> Result<
        Option<FourChannelRadioData>,
        TransferError<<SPI as spi::ErrorType>::Error, <CE as digital::ErrorType>::Error>,
    > {
        if self.radio.read(&mut self.latest_packet).await? {
            let radio_data = FourChannelRadioData {
                throttle: self.latest_packet[0],
                yaw: syma_convert_to_signed(self.latest_packet[1]),
                pitch: -syma_convert_to_signed(self.latest_packet[2]),
                roll: syma_convert_to_signed(self.latest_packet[3]),
            };

            // Hop to the next channel after seeing two packets. This does
            // not implement any "catch-up" algorithm if we get out of sync
            // with the transmitter because this naive implementation re-syncs
            // relatively quickly.
            self.packets_received_on_current_channel += 1;
            if self.packets_received_on_current_channel == 2 {
                self.current_channel_idx = (self.current_channel_idx + 1) % NUM_DATA_CHANNELS;
                self.radio
                    .set_channel(DATA_CHANNELS[self.current_channel_idx])
                    .await?;

                self.packets_received_on_current_channel = 0;
            }
            Ok(Some(radio_data))
        } else {
            Ok(None)
        }
    }
}

fn syma_convert_to_signed(input: u8) -> i8 {
    let ret = (input & 0b0111_1111) as i8;

    if (input & 0b1000_0000) == 0 {
        -ret
    } else {
        ret
    }
}
