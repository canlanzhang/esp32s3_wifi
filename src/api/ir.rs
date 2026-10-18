use anyhow::Result;
use esp_idf_hal::io::Write;
use esp_idf_svc::http::server::{EspHttpServer, Method};

use crate::ir_control::{IrCommand, IrSender};

pub fn register(
    server: &mut EspHttpServer<'static>,
    ir_sender: IrSender,
) -> Result<()> {
    let toggle_sender = ir_sender.clone();

    server.fn_handler("/ir/toggle", Method::Get, move |req| {
        let body = match toggle_sender.send(IrCommand {
            address: 0x80,
            command: 0x01,
        }) {
            Ok(()) => "IR command queued",
            Err(e) => {
                log::error!("IR command queue failed: {:?}", e);
                "IR command queue failed"
            }
        };

        let mut response = req.into_ok_response()?;
        response.write_all(body.as_bytes())?;
        Ok::<(), anyhow::Error>(())
    })?;

    server.fn_handler("/ir/send", Method::Get, move |req| {
        let uri = req.uri().to_owned();

        let mut address = None;
        let mut command = None;

        if let Some((_, query)) = uri.split_once('?') {
            for pair in query.split('&') {
                if let Some((key, value)) = pair.split_once('=') {
                    let parsed = if let Some(hex) = value
                        .strip_prefix("0x")
                        .or_else(|| value.strip_prefix("0X"))
                    {
                        u8::from_str_radix(hex, 16).ok()
                    } else {
                        value.parse::<u8>().ok()
                    };

                    match key {
                        "address" => address = parsed,
                        "command" => command = parsed,
                        _ => {}
                    }
                }
            }
        }

        let body = match (address, command) {
            (Some(address), Some(command)) => {
                match ir_sender.send(IrCommand { address, command }) {
                    Ok(()) => "IR command queued",
                    Err(e) => {
                        log::error!("IR command queue failed: {:?}", e);
                        "IR command queue failed"
                    }
                }
            }
            _ => "Invalid parameters. Example: /ir/send?address=128&command=1",
        };

        let mut response = req.into_ok_response()?;
        response.write_all(body.as_bytes())?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}