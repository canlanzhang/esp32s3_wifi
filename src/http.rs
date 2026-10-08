use anyhow::Result;


use esp_idf_hal::io::Write;


use esp_idf_svc::http::server::{
    Configuration as HttpConfiguration,
    EspHttpServer,
    Method,
};


use crate::led::Led;



pub fn start_http_server(

    led: Led,

    ip: String,

) -> Result<EspHttpServer<'static>> {



    let mut server =
        EspHttpServer::new(
            &HttpConfiguration::default()
        )?;





    // ==========================
    // 首页
    // ==========================

    server.fn_handler(
        "/",
        Method::Get,
        |req| {


            let mut response =
                req.into_ok_response()?;


            response.write_all(
                b"ESP32-S3 HTTP OK"
            )?;



            Ok::<(), anyhow::Error>(())

        }

    )?;







    // ==========================
    // LED ON
    // ==========================


    {

        let led =
            led.clone();


        server.fn_handler(
            "/led/on",
            Method::Get,
            move |req| {


                led.on()?;



                let mut response =
                    req.into_ok_response()?;


                response.write_all(
                    b"LED ON"
                )?;



                Ok::<(), anyhow::Error>(())

            }

        )?;

    }








    // ==========================
    // LED OFF
    // ==========================


    {

        let led =
            led.clone();



        server.fn_handler(
            "/led/off",
            Method::Get,
            move |req| {



                led.off()?;




                let mut response =
                    req.into_ok_response()?;


                response.write_all(
                    b"LED OFF"
                )?;




                Ok::<(), anyhow::Error>(())

            }

        )?;

    }









    // ==========================
    // API STATUS
    // ==========================


    {

        let led =
            led.clone();



        server.fn_handler(
            "/api/status",
            Method::Get,
            move |req| {



                let status =
                    if led.is_on() {

                        "on"

                    } else {

                        "off"

                    };





                let json =
                    format!(
r#"{{
    "device":"ESP32-S3",
    "wifi":true,
    "ip":"{}",
    "led":"{}"
}}"#,
                    ip,
                    status
                );





                let mut response =
                    req.into_ok_response()?;




                response.write_all(
                    json.as_bytes()
                )?;





                Ok::<(), anyhow::Error>(())

            }

        )?;

    }





    Ok(server)

}