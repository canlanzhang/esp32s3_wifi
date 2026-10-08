use anyhow::Result;

use esp_idf_hal::io::Write;

use esp_idf_svc::http::server::{EspHttpServer, Method};

use crate::led::Led;

pub fn register(server: &mut EspHttpServer<'static>, led: Led, ip: String) -> Result<()> {
    server.fn_handler("/api/status", Method::Get, move |req| {
        let status = if led.is_on() { "on" } else { "off" };

        let json = format!(
            r#"{{
    "device":"ESP32-S3",
    "wifi":true,
    "ip":"{}",
    "led":"{}"
}}"#,
            ip, status
        );

        let mut response = req.into_ok_response()?;

        response.write_all(json.as_bytes())?;

        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
