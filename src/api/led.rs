use anyhow::Result;

use esp_idf_hal::io::Write;

use esp_idf_svc::http::server::{
    EspHttpServer,
    Method,
};

use crate::led::Led;

pub fn register(
    server: &mut EspHttpServer<'static>,
    led: Led,
) -> Result<()> {
    // ==========================
    // LED API
    //
    // /api/led
    //              查询状态
    //
    // /api/led?state=on
    //              开灯
    //
    // /api/led?state=off
    //              关灯
    //
    // /api/led?state=toggle
    //              切换
    // ==========================

    {
        let led = led.clone();

        server.fn_handler(
            "/api/led",
            Method::Get,
            move |req| {
                // ==========================
                // 获取 URL
                // ==========================

                let uri = req.uri();

                // ==========================
                // 解析 state
                // ==========================

                let mut state: Option<&str> = None;

                if let Some(pos) = uri.find('?') {
                    let query = &uri[pos + 1..];

                    for param in query.split('&') {
                        let mut parts =
                            param.splitn(2, '=');

                        let key =
                            parts.next().unwrap_or("");

                        let value =
                            parts.next().unwrap_or("");

                        if key == "state" {
                            state = Some(value);
                            break;
                        }
                    }
                }

                // ==========================
                // 执行 LED 操作
                // ==========================

                match state {
                    // --------------------------
                    // ON
                    // --------------------------

                    Some("on") => {
                        led.on()?;
                    }

                    // --------------------------
                    // OFF
                    // --------------------------

                    Some("off") => {
                        led.off()?;
                    }

                    // --------------------------
                    // TOGGLE
                    // --------------------------

                    Some("toggle") => {
                        if led.is_on() {
                            led.off()?;
                        } else {
                            led.on()?;
                        }
                    }

                    // --------------------------
                    // 未知操作
                    // --------------------------

                    Some(_) => {
                        let json = r#"{
    "success":false,
    "error":"invalid_state",
    "message":"state must be on, off or toggle"
}"#;

                        let mut response =
                            req.into_response(
                                400,
                                Some("Bad Request"),
                                &[
                                    (
                                        "Content-Type",
                                        "application/json",
                                    ),
                                ],
                            )?;

                        response.write_all(
                            json.as_bytes()
                        )?;

                        return Ok::<(), anyhow::Error>(());
                    }

                    // --------------------------
                    // 没有 state
                    //
                    // 只查询状态
                    // --------------------------

                    None => {}
                }

                // ==========================
                // 获取当前状态
                // ==========================

                let status =
                    if led.is_on() {
                        "on"
                    } else {
                        "off"
                    };

                // ==========================
                // 返回 JSON
                // ==========================

                let json = format!(
                    r#"{{
    "success":true,
    "led":"{}"
}}"#,
                    status
                );

                // ==========================
                // HTTP Response
                // ==========================

                let mut response =
                    req.into_response(
                        200,
                        Some("OK"),
                        &[
                            (
                                "Content-Type",
                                "application/json",
                            ),
                        ],
                    )?;

                response.write_all(
                    json.as_bytes()
                )?;

                Ok::<(), anyhow::Error>(())
            },
        )?;
    }

    Ok(())
}