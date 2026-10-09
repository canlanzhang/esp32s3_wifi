mod api;
mod http;
mod led;
mod sht31;
mod wifi;

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

    // ==========================
    // LED GPIO48
    // ==========================

    let led = led::Led::new(PinDriver::output(peripherals.pins.gpio48)?);

    // ==========================
    // SHT31 I2C
    //
    // SDA GPIO8
    // SCL GPIO9
    // ==========================

    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio8,
        peripherals.pins.gpio9,
        &I2cConfig::new(),
    )?;

    let sht31 = sht31::Sht31Sensor::new(i2c)?;

    // ==========================
    // WIFI
    // ==========================

    let ip = wifi::connect_wifi(peripherals.modem)?;

    // ==========================
    // HTTP
    // ==========================

    let server = http::start_http_server(
    led.clone(),
    sht31.clone(),
    ip.to_string()
)?;

    info!("HTTP server started");

    loop {
        FreeRtos::delay_ms(1000);
    }
}
