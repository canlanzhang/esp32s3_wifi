use anyhow::Result;

use esp_idf_svc::http::server::{Configuration, EspHttpServer};

use crate::{led::Led, sht31::Sht31Sensor};

use crate::api;

pub fn start_http_server(
    led: Led,

    sht31: Sht31Sensor,

    ip: String,
) -> Result<EspHttpServer<'static>> {
    let mut server = EspHttpServer::new(&Configuration::default())?;

    api::status::register(&mut server, led.clone(), ip)?;

    api::led::register(&mut server, led)?;

    api::sensor::register(&mut server, sht31)?;

    Ok(server)
}
