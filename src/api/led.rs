use anyhow::Result;

use esp_idf_hal::io::Write;

use esp_idf_svc::http::server::{EspHttpServer, Method};

use crate::led::Led;

pub fn register(server: &mut EspHttpServer<'static>, led: Led) -> Result<()> {
    // ==========================
    // LED ON
    // ==========================

    {
        let led = led.clone();

        server.fn_handler("/led/on", Method::Get, move |req| {
            led.on()?;

            let mut response = req.into_ok_response()?;

            response.write_all(b"LED ON")?;

            Ok::<(), anyhow::Error>(())
        })?;
    }

    // ==========================
    // LED OFF
    // ==========================

    {
        let led = led.clone();

        server.fn_handler("/led/off", Method::Get, move |req| {
            led.off()?;

            let mut response = req.into_ok_response()?;

            response.write_all(b"LED OFF")?;

            Ok::<(), anyhow::Error>(())
        })?;
    }

    // ==========================
    // LED STATUS
    // ==========================

    {
        let led = led.clone();

        server.fn_handler("/api/led", Method::Get, move |req| {
            let status = if led.is_on() { "on" } else { "off" };

            let json = format!(
                r#"{{
    "led":"{}"
}}"#,
                status
            );

            let mut response = req.into_ok_response()?;

            response.write_all(json.as_bytes())?;

            Ok::<(), anyhow::Error>(())
        })?;
    }

    Ok(())
}
