
use anyhow::Result;

use esp_idf_hal::gpio::OutputPin;
use esp_idf_hal::rmt::config::{
    CarrierConfig,
    DutyPercent,
    TransmitConfig,
    TxChannelConfig,
};
use esp_idf_hal::rmt::encoder::CopyEncoder;
use esp_idf_hal::rmt::{
    PinState,
    Pulse,
    PulseTicks,
    RmtChannel,
    Symbol,
    TxChannelDriver,
};
use esp_idf_hal::units::Hertz;

pub const IR_TX_GPIO: u8 = 1;

const RMT_RESOLUTION: Hertz = Hertz(1_000_000);
const IR_CARRIER_HZ: Hertz = Hertz(38_000);

// 根据原设备的接收日志调整
const NEC_MARK_US: u16 = 600;
const NEC_ZERO_SPACE_US: u16 = 525;
const NEC_ONE_SPACE_US: u16 = 1660;

// 引导码先使用标准 NEC 时序。
// 需要时再根据原装遥控器的完整引导码日志调整。
const NEC_HEADER_MARK_US: u16 = 9000;
const NEC_HEADER_SPACE_US: u16 = 4500;

pub struct IrTx<'d> {
    tx: TxChannelDriver<'d>,
    encoder: CopyEncoder,
}

impl<'d> IrTx<'d> {
    pub fn new(pin: impl OutputPin + 'd) -> Result<Self> {
        log::info!(
            "IR TX: creating RMT channel on GPIO{}",
            IR_TX_GPIO
        );

        let mut tx = TxChannelDriver::new(
            pin,
            &TxChannelConfig {
                resolution: RMT_RESOLUTION,
                ..Default::default()
            },
        )?;

        // 38 kHz 载波，33% 占空比
        let carrier = CarrierConfig {
            frequency: IR_CARRIER_HZ,
            duty_cycle: DutyPercent::new(0.33)?,
            polarity_active_low: false,
            always_on: false,
            ..Default::default()
        };

        tx.apply_carrier(Some(&carrier))?;

        let encoder = CopyEncoder::new()?;

        log::info!("IR TX: carrier configured at 38 kHz");

        Ok(Self { tx, encoder })
    }

    /// 发送 NEC 32 位命令。
    ///
    /// 数据顺序：地址、地址反码、命令、命令反码。
    /// 每个字节按最低有效位优先发送。
    pub fn send_nec(
        &mut self,
        address: u8,
        command: u8,
    ) -> Result<()> {
        let mut symbols: Vec<Symbol> = Vec::with_capacity(34);

        // NEC 引导码：9 ms mark + 4.5 ms space
        symbols.push(Symbol::new(
            Pulse::new(
                PinState::High,
                PulseTicks::new(NEC_HEADER_MARK_US)?,
            ),
            Pulse::new(
                PinState::Low,
                PulseTicks::new(NEC_HEADER_SPACE_US)?,
            ),
        ));

        let data = [address, !address, command, !command];

        for byte in data {
            for bit in 0..8 {
                let one = (byte & (1 << bit)) != 0;

                let space_us = if one {
                    NEC_ONE_SPACE_US
                } else {
                    NEC_ZERO_SPACE_US
                };

                symbols.push(Symbol::new(
                    Pulse::new(
                        PinState::High,
                        PulseTicks::new(NEC_MARK_US)?,
                    ),
                    Pulse::new(
                        PinState::Low,
                        PulseTicks::new(space_us)?,
                    ),
                ));
            }
        }

        // 最后一个 mark。后续空闲电平由 RMT 停止状态决定。
        symbols.push(Symbol::new(
            Pulse::new(
                PinState::High,
                PulseTicks::new(NEC_MARK_US)?,
            ),
            Pulse::new(
                PinState::Low,
                PulseTicks::new(NEC_ZERO_SPACE_US)?,
            ),
        ));

        log::info!(
            "IR TX: sending NEC address=0x{:02X}, command=0x{:02X}",
            address,
            command
        );

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
