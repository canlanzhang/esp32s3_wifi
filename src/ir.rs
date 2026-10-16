use anyhow::Result;

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::rmt::{
    config::{ReceiveConfig, RxChannelConfig},
    RxChannelDriver,
    Symbol,
};
use esp_idf_hal::units::Hertz;

use std::time::Duration;

/// 红外接收模块 OUT 接 GPIO2
pub const IR_GPIO: u8 = 2;

/// RMT 分辨率：1 MHz
///
/// 1 tick = 1 us
const RMT_RESOLUTION: Hertz = Hertz(1_000_000);

/// 一帧最多保存 200 个 Symbol
const RAW_BUFFER_SIZE: usize = 200;

/// 启动红外接收任务
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

/// 红外接收循环
fn receive_loop(pin: impl InputPin) -> Result<()> {
    // ==========================
    // 创建 RMT RX
    // ==========================

    let mut rx = RxChannelDriver::new(
        pin,
        &RxChannelConfig {
            // 1 MHz
            //
            // 1 tick = 1us
            resolution: RMT_RESOLUTION,

            ..Default::default()
        },
    )?;

    log::info!(
        "IR receiver ready on GPIO{}, resolution={} Hz",
        IR_GPIO,
        1_000_000
    );

    // ==========================
    // 接收配置
    // ==========================

    let rx_config = ReceiveConfig {
        // 不过滤短脉冲
        signal_range_min: Duration::ZERO,

        // 3ms
        //
        // 注意：
        // ESP-IDF RMT RX 在当前配置下最大约 3.2767ms。
        //
        // 3ms 足够覆盖绝大多数红外数据脉冲：
        // 560us
        // 1690us
        //
        // 但是不能直接用 9ms 作为一个 pulse。
        signal_range_max: Duration::from_millis(3),

        // 1 秒等待
        timeout: None,

        ..Default::default()
    };

    // ==========================
    // 开始接收
    // ==========================

    loop {
        let mut buffer = [Symbol::default(); RAW_BUFFER_SIZE];

        let count = match rx.receive(&mut buffer, &rx_config) {
            Ok(count) => count,

            Err(e) => {
                log::warn!("IR receive error: {:?}", e);
                continue;
            }
        };

        if count == 0 {
            continue;
        }

        log::info!("========== IR FRAME ==========");

        for (i, symbol) in buffer[..count].iter().enumerate() {
            let p0 = symbol.level0();
            let p1 = symbol.level1();

            // PulseTicks 内部字段是 private，
            // 必须使用 ticks()。
            let ticks0 = p0.ticks.ticks();
            let ticks1 = p1.ticks.ticks();

            let duration0 = p0.ticks.duration(RMT_RESOLUTION);
            let duration1 = p1.ticks.duration(RMT_RESOLUTION);

log::info!(
    "#{:03}: {:?} {:>6}us ({:>6} ticks) | {:?} {:>6}us ({:>6} ticks)",
    i,
    p0.pin_state,
    duration0.as_micros(),
    ticks0,
    p1.pin_state,
    duration1.as_micros(),
    ticks1,
);
        }

        log::info!(
            "========== IR FRAME END: {} symbols ==========",
            count
        );
    }
}