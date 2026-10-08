mod wifi;
mod http;
mod led;


use anyhow::Result;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;

use log::info;



fn main() -> Result<()> {


    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();


    info!("ESP32-S3 start");



    let peripherals =
        Peripherals::take()?;



    // ==========================
    // LED GPIO48
    // ==========================

    let led =
        led::Led::new(
            PinDriver::output(
                peripherals.pins.gpio48
            )?
        );



    // ==========================
    // WIFI
    // ==========================

    let ip =
        wifi::connect_wifi(
            peripherals.modem
        )?;



    // ==========================
    // HTTP SERVER
    // ==========================

    let _server =
        http::start_http_server(
            led.clone(),
            ip.to_string(),
        )?;



    info!("HTTP server started");



    loop {

        FreeRtos::delay_ms(1000);

    }

}