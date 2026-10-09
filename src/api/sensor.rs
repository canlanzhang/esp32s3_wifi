use anyhow::Result;

use esp_idf_hal::io::Write;

use esp_idf_svc::http::server::{EspHttpServer, Method};

use crate::sht31::Sht31Sensor;

pub fn register(server: &mut EspHttpServer<'static>, sht31: Sht31Sensor) -> Result<()> {
    server.fn_handler("/api/sht31", Method::Get, move |req| {
        let (temp, hum) = sht31.read()?;

        let temp_c = (temp - 32.0) * 5.0 / 9.0;

        let json = format!(
            r#"{{
    "temperature":{:.2},
    "humidity":{:.2}
}}"#,
            temp_c, hum
        );

        let mut response = req.into_ok_response()?;

        response.write_all(json.as_bytes())?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
