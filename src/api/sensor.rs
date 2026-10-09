use anyhow::Result;

use esp_idf_hal::io::Write;

use esp_idf_svc::http::server::{EspHttpServer, Method};

use crate::sht31::Sht31Sensor;

pub fn register(server: &mut EspHttpServer<'static>, sht31: Sht31Sensor) -> Result<()> {
    server.fn_handler("/api/sht31", Method::Get, move |req| {

    let (temp, hum) = sht31.read()?;


    let json = format!(
        r#"{{
    "temperature":{:.2},
    "humidity":{:.2}
}}"#,
        temp,
        hum
    );


    let mut response = req.into_response(
        200,
        Some("OK"),
        &[("Content-Type","application/json")]
    )?;


    response.write_all(json.as_bytes())?;


    Ok::<(), anyhow::Error>(())

})?;
    Ok(())
}
