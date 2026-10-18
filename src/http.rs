use anyhow::Result;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};

use crate::api;
use crate::board_led::Board_Led;
use crate::ir_control::IrSender;
use crate::sht31::Sht31Sensor;

pub fn start_http_server(
    led: Board_Led,
    sht31: Sht31Sensor,
    ir_sender: IrSender,
    ip: String,
) -> Result<EspHttpServer<'static>> {
    let mut server = EspHttpServer::new(&Configuration::default())?;

    api::status::register(&mut server, led.clone(), ip)?;
    api::board_led::register(&mut server, led)?;
    api::sensor::register(&mut server, sht31)?;
    api::ir::register(&mut server, ir_sender)?;

    Ok(server)
}