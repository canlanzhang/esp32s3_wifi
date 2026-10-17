mod api;
mod http;
mod ir;
mod board_led;
mod sht31;
mod wifi;
mod ir_led;
mod ir_tx;

use anyhow::Result;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use log::info;

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("ESP32-S3 start");

    let peripherals = Peripherals::take()?;

    // 红外发射器：GPIO1，先初始化，不立即发送
let mut ir_tx = ir_tx::IrTx::new(peripherals.pins.gpio1)?;

// 其他初始化：GPIO、SHT31、接收端、Wi-Fi、HTTP
// 全部完成后：
FreeRtos::delay_ms(2000);
ir_tx.send_nec(0x80, 0x01)?;

    // 外部红外指示灯：GPIO17
    let ir_led = ir_led::IrLed::new(
        PinDriver::output(peripherals.pins.gpio17)?
    );

    // 板载 LED：GPIO48
    let led = board_led::Board_Led::new(
        PinDriver::output(peripherals.pins.gpio48)?
    );

    // SHT31：SDA GPIO8，SCL GPIO9
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &I2cConfig::new(),
    )?;

    let sht31 = sht31::Sht31Sensor::new(i2c)?;

    // 红外接收：GPIO2
    ir::start(peripherals.pins.gpio2, ir_led.clone())?;

    // 连接 Wi-Fi
    let ip = wifi::connect_wifi(peripherals.modem)?;
    info!("Wi-Fi connected: {}", ip);

    // 启动 HTTP 服务
    let _server = http::start_http_server(
        led.clone(),
        sht31.clone(),
        ip.to_string(),
    )?;

    info!("HTTP server started");
    info!("System initialization complete");

    // 完整启动后等待 2 秒，再发送一次红外指令
    FreeRtos::delay_ms(2000);

    info!("Sending IR command");
    ir_tx.send_nec(0x80, 0x01)?;
    info!("IR command sent");

    // 主循环
    loop {
        FreeRtos::delay_ms(1000);
    }
}