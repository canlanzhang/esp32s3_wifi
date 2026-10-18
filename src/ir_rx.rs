use anyhow::Result;
use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::rmt::{
    config::{ReceiveConfig, RxChannelConfig},
    RxChannelDriver, Symbol,
};
use esp_idf_hal::units::Hertz;
use std::time::Duration;

use crate::ir_led::IrLed;

pub const IR_GPIO: u8 = 2;

const RMT_RESOLUTION: Hertz = Hertz(1_000_000);
const RAW_BUFFER_SIZE: usize = 200;

// NEC 数据位的高电平时长范围，单位 us
const ZERO_MIN: u64 = 350;
const ZERO_MAX: u64 = 900;
const ONE_MIN: u64 = 1200;
const ONE_MAX: u64 = 2000;

// 当前遥控器按键映射
const IR_ADDRESS: u8 = 0x80;
const IR_TOGGLE_COMMAND: u8 = 0x01;

pub fn start(pin: impl InputPin + 'static, led: IrLed) -> Result<()> {
    std::thread::Builder::new()
        .name("ir_rx".into())
        .stack_size(4096)
        .spawn(move || {
            if let Err(e) = receive_loop(pin, led) {
                log::error!("IR receiver error: {:?}", e);
            }
        })?;

    log::info!("IR receiver task started on GPIO{}", IR_GPIO);
    Ok(())
}

fn receive_loop(pin: impl InputPin, led: IrLed) -> Result<()> {
    let mut rx = RxChannelDriver::new(
        pin,
        &RxChannelConfig {
            resolution: RMT_RESOLUTION,
            ..Default::default()
        },
    )?;

    log::info!(
        "IR receiver ready on GPIO{}, resolution=1000000 Hz",
        IR_GPIO
    );

    let rx_config = ReceiveConfig {
        signal_range_min: Duration::ZERO,
        signal_range_max: Duration::from_millis(15),
        timeout: None,
        ..Default::default()
    };

    loop {
        let mut buffer = [Symbol::default(); RAW_BUFFER_SIZE];

        let count = match rx.receive(&mut buffer, &rx_config) {
            Ok(count) => count,
            Err(e) => {
                log::warn!("IR receive error: {:?}", e);
                continue;
            }
        };

        // 忽略空帧
        if count == 0 {
            continue;
        }

        let symbols = &buffer[..count];

        let has_signal = symbols.iter().any(|symbol| {
            symbol.level0().ticks.duration(RMT_RESOLUTION).as_micros() > 0
                || symbol.level1().ticks.duration(RMT_RESOLUTION).as_micros() > 0
        });

        if !has_signal {
            continue;
        }

        log::info!("========== IR FRAME: {} symbols ==========", count);

        // 保留完整红外原始波形日志，便于后续匹配其他遥控器
        for (i, symbol) in symbols.iter().enumerate() {
            let p0 = symbol.level0();
            let p1 = symbol.level1();

            let us0 = p0.ticks.duration(RMT_RESOLUTION).as_micros();
            let us1 = p1.ticks.duration(RMT_RESOLUTION).as_micros();

            log::info!(
                "#{:03}: {:?} {:>5}us ({} ticks) | {:?} {:>5}us ({} ticks)",
                i,
                p0.pin_state,
                us0,
                p0.ticks.ticks(),
                p1.pin_state,
                us1,
                p1.ticks.ticks(),
            );
        }

        // NEC 解码；无论是否识别为已配置按键，都保留调试信息
        match decode_nec(symbols) {
            Some((address, command)) => {
                log::info!(
                    "IR decoded: protocol=NEC address=0x{:02X} command=0x{:02X}",
                    address,
                    command
                );

                // 当前遥控器的 Toggle 按键
                if address == IR_ADDRESS && command == IR_TOGGLE_COMMAND {
                    let was_on = led.is_on();

                    let result = if was_on {
                        led.off()
                    } else {
                        led.on()
                    };

                    match result {
                        Ok(()) => {
                            log::info!(
                                "IR toggle: address=0x{:02X} command=0x{:02X}, LED {} -> {}",
                                address,
                                command,
                                if was_on { "ON" } else { "OFF" },
                                if was_on { "OFF" } else { "ON" }
                            );
                        }
                        Err(e) => {
                            log::error!("IR LED toggle failed: {:?}", e);
                        }
                    }
                } else {
                    // 未配置的按键也记录，方便后续添加映射
                    log::info!(
                        "IR command not mapped: address=0x{:02X} command=0x{:02X}",
                        address,
                        command
                    );
                }
            }
            None => {
                log::warn!("IR decode failed; raw waveform printed above");
            }
        }

        log::info!("========== IR FRAME END ==========");
    }
}

/// 根据 RMT Symbol 的电平时长解析 NEC 32 位数据。
/// 地址反码和命令反码都校验通过后才返回结果。
fn decode_nec(symbols: &[Symbol]) -> Option<(u8, u8)> {
    if symbols.len() < 32 {
        return None;
    }

    let mut data: u32 = 0;

    for (i, symbol) in symbols.iter().take(32).enumerate() {
        let p0 = symbol.level0();
        let p1 = symbol.level1();

        let low_us = p0.ticks.duration(RMT_RESOLUTION).as_micros() as u64;
        let high_us = p1.ticks.duration(RMT_RESOLUTION).as_micros() as u64;

        // NEC 每一位通常包含约 560us 的低电平
        if !(ZERO_MIN..=ZERO_MAX).contains(&low_us) {
            return None;
        }

        // 根据高电平时长判断 0 或 1
        let bit = if (ZERO_MIN..=ZERO_MAX).contains(&high_us) {
            0u32
        } else if (ONE_MIN..=ONE_MAX).contains(&high_us) {
            1u32
        } else {
            return None;
        };

        // NEC 数据按最低有效位优先传输
        data |= bit << i;
    }

    let address = (data & 0xFF) as u8;
    let address_inverse = ((data >> 8) & 0xFF) as u8;
    let command = ((data >> 16) & 0xFF) as u8;
    let command_inverse = ((data >> 24) & 0xFF) as u8;

    // 校验地址反码与命令反码
    if address ^ address_inverse != 0xFF {
        return None;
    }

    if command ^ command_inverse != 0xFF {
        return None;
    }

    Some((address, command))
}