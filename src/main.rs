
mod api;
mod http;
mod ir_rx;
mod board_led;
mod sht31;
mod wifi;
mod ir_led;
mod ir_tx;
mod ir_control;

use anyhow::Result;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use log::info;
use std::sync::mpsc::TryRecvError;

fn main() -> Result<()> {
    // 初始化 ESP-IDF
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("ESP32-S3 start");

    // 获取外设
    let peripherals = Peripherals::take()?;

    // ========================================
    // 红外发射器：GPIO1
    // 由主线程持有，避免跨线程共享 RMT Encoder
    // ========================================
    let mut ir_tx = ir_tx::IrTx::new(peripherals.pins.gpio1)?;

    // 创建红外命令通道
    let (ir_sender, ir_receiver) = ir_control::channel();

    // ========================================
    // 红外指示灯：GPIO17
    // ========================================
    let ir_led = ir_led::IrLed::new(
        PinDriver::output(peripherals.pins.gpio17)?,
    );

    // ========================================
    // 板载 LED：GPIO48
    // ========================================
    let led = board_led::Board_Led::new(
        PinDriver::output(peripherals.pins.gpio48)?,
    );

    // ========================================
    // SHT31 温湿度传感器
    // SDA：GPIO8
    // SCL：GPIO9
    // ========================================
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio41,
        peripherals.pins.gpio42,
        &I2cConfig::new(),
    )?;

    let sht31 = sht31::Sht31Sensor::new(i2c)?;

    // ========================================
    // 启动红外接收任务：GPIO2
    // ========================================
    ir_rx::start(
        peripherals.pins.gpio2,
        ir_led.clone(),
    )?;

    // ========================================
    // 连接 Wi-Fi
    // ========================================
    let ip = wifi::connect_wifi(peripherals.modem)?;

    info!("Wi-Fi connected: {}", ip);

    // ========================================
    // 启动 HTTP 服务
    // ========================================
    let _server = http::start_http_server(
        led.clone(),
        sht31.clone(),
        ir_sender,
        ip.to_string(),
    )?;

    info!("HTTP server started");
    info!("System initialization complete");
    info!("IR toggle URL: http://{}/ir/toggle", ip);
    info!(
        "IR send URL: http://{}/ir/send?address=128&command=1",
        ip
    );

    // ========================================
    // 主循环：处理 HTTP 提交的红外命令
    // ========================================
    loop {
        match ir_receiver.try_recv() {
            Ok(cmd) => {
                info!(
                    "Sending IR command: address=0x{:02X}, command=0x{:02X}",
                    cmd.address,
                    cmd.command
                );

                if let Err(e) = ir_tx.send_nec(cmd.address, cmd.command) {
                    log::error!("IR send failed: {:?}", e);
                } else {
                    info!("IR command transmission completed");
                }
            }

            Err(TryRecvError::Empty) => {
                // 当前没有待发送的红外命令
            }

            Err(TryRecvError::Disconnected) => {
                log::error!("IR command channel disconnected");
            }
        }

        FreeRtos::delay_ms(10);
    }
}
