
use anyhow::Result;

use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::rmt::config::{
    TransmitConfig,
    TxChannelConfig,
};
use esp_idf_hal::rmt::encoder::CopyEncoder;
use esp_idf_hal::rmt::{
    PinState,
    Pulse,
    PulseTicks,
    Symbol,
    TxChannelDriver,
};
use esp_idf_hal::units::Hertz;

pub const IR_TX_GPIO: u8 = 1;

const RMT_RESOLUTION: Hertz = Hertz(1_000_000);

pub struct IrTx<'d> {
    tx: TxChannelDriver<'d>,
    encoder: CopyEncoder,
}

impl<'d> IrTx<'d> {
    pub fn new(pin: impl OutputPin + 'd) -> Result<Self> {
        log::info!("IR TX: creating RMT channel");

        let tx = TxChannelDriver::new(
            pin,
            &TxChannelConfig {
                resolution: RMT_RESOLUTION,
                ..Default::default()
            },
        )?;

        log::info!("IR TX: RMT channel created");

        let encoder = CopyEncoder::new()?;

        log::info!("IR TX: encoder created");

        Ok(Self { tx, encoder })
    }

    /// 发送标准 NEC 32 位命令。
    pub fn send_nec(
        &mut self,
        address: u8,
        command: u8,
    ) -> Result<()> {
        let mut symbols: Vec<Symbol> = Vec::with_capacity(34);

        // NEC 引导码：9 ms + 4.5 ms
        symbols.push(Symbol::new(
            Pulse::new(PinState::High, PulseTicks::new(9000)?),
            Pulse::new(PinState::Low, PulseTicks::new(4500)?),
        ));

        // 地址、地址反码、命令、命令反码
        let data = [address, !address, command, !command];

        for byte in data {
            for bit in 0..8 {
                let one = (byte & (1 << bit)) != 0;
                let space_us = if one { 1690 } else { 560 };

                symbols.push(Symbol::new(
                    Pulse::new(PinState::High, PulseTicks::new(560)?),
                    Pulse::new(PinState::Low, PulseTicks::new(space_us)?),
                ));
            }
        }

        // 结束脉冲
        symbols.push(Symbol::new(
            Pulse::new(PinState::High, PulseTicks::new(560)?),
            Pulse::new(PinState::Low, PulseTicks::new(560)?),
        ));

        log::info!("IR TX: sending NEC command");

        unsafe {
            self.tx.start_send(
                &mut self.encoder,
                &symbols,
                &TransmitConfig::default(),
            )?;
        }

        Ok(())
    }
}
