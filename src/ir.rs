
use anyhow::Result;
use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::rmt::{
    config::{ReceiveConfig, RxChannelConfig},
    RxChannelDriver, Symbol,
};
use esp_idf_hal::units::Hertz;
use std::time::Duration;

pub const IR_GPIO: u8 = 2;

const RMT_RESOLUTION: Hertz = Hertz(1_000_000);
const RAW_BUFFER_SIZE: usize = 200;

// NEC 数据位的高电平时长范围，单位 us
const ZERO_MIN: u64 = 350;
const ZERO_MAX: u64 = 900;
const ONE_MIN: u64 = 1200;
const ONE_MAX: u64 = 2000;

pub fn start(pin: impl InputPin + 'static) -> Result<()> {
    std::thread::Builder::new()
        .name("ir_rx".into())
        .stack_size(4096)
        .spawn(move || {
            if let Err(e) = receive_loop(pin) {
                log::error!("IR receiver error: {:?}", e);
            }
        })?;

    log::info!("IR receiver task started on GPIO{}", IR_GPIO);
    Ok(())
}

fn receive_loop(pin: impl InputPin) -> Result<()> {
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
        signal_range_max: Duration::from_millis(3),
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

        // 忽略空帧和明显不完整的帧
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

        // 打印完整波形，用于确认实际电平顺序和时序
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

        // 尝试解析 NEC 32 位数据
        match decode_nec(symbols) {
            Some((address, command)) => {
                log::info!(
                    "IR decoded: protocol=NEC address=0x{:02X} command=0x{:02X}",
                    address,
                    command
                );
            }
            None => {
                log::warn!("IR decode failed; raw waveform printed above");
            }
        }

        log::info!("========== IR FRAME END ==========");
    }
}

/// 根据每个 Symbol 的 level0 时长，尝试解析 NEC 数据。
/// 只有地址反码和命令反码均通过校验，才返回结果。

fn decode_nec(symbols: &[Symbol]) -> Option<(u8, u8)> {
    // NEC 数据至少需要 32 个 Symbol
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
        if !(350..=900).contains(&low_us) {
            return None;
        }

        // 读取高电平时长来判断数据位
        let bit = if (350..=900).contains(&high_us) {
            0u32
        } else if (1200..=2000).contains(&high_us) {
            1u32
        } else {
            return None;
        };

        // NEC 按最低有效位优先传输
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
