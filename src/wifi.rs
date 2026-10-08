use anyhow::Result;

use esp_idf_hal::delay::FreeRtos;

use esp_idf_svc::eventloop::EspSystemEventLoop;

use esp_idf_svc::nvs::EspDefaultNvsPartition;

use esp_idf_svc::wifi::{
    AuthMethod,
    BlockingWifi,
    ClientConfiguration,
    Configuration,
    EspWifi,
};



const WIFI_SSID:&str="HUAWEI-LL";

const WIFI_PASS:&str="passworld";



fn wifi_config()->Configuration
{

    Configuration::Client(

        ClientConfiguration{

            ssid:
                WIFI_SSID
                .try_into()
                .unwrap(),

            password:
                WIFI_PASS
                .try_into()
                .unwrap(),

            auth_method:
                AuthMethod::WPA2Personal,


            /*
                不固定频道

                让AP决定
            */

            channel:
                None,


            ..Default::default()

        }

    )

}





pub fn connect_wifi(

    modem:
        esp_idf_hal::modem::Modem

)
->Result<std::net::Ipv4Addr>
{


    let sysloop =
        EspSystemEventLoop::take()?;


    let nvs =
        EspDefaultNvsPartition::take()?;



    let mut wifi =
        BlockingWifi::wrap(

            EspWifi::new(

                modem,

                sysloop.clone(),

                Some(nvs),

            )?,

            sysloop,

        )?;



    /*
        start
    */

    wifi.start()?;


    log::info!(
        "wifi started"
    );



    FreeRtos::delay_ms(
        300
    );



    wifi.set_configuration(
        &wifi_config()
    )?;



    /*
        第一次连接
    */


    let mut connected=false;


    for attempt in 1..=3
    {


        log::info!(
            "wifi connect attempt {}",
            attempt
        );



        match wifi.connect()
        {


            Ok(_)=>{


                connected=true;


                log::info!(
                    "wifi connected"
                );


                break;

            }



            Err(e)=>{


                log::warn!(
                    "wifi failed {:?}",
                    e
                );



                /*
                    第一次失败

                    重启驱动
                */

                if attempt==1
                {


                    log::warn!(
                        "wifi driver reset"
                    );



                    let _=
                        wifi.disconnect();



                    let _=
                        wifi.stop();



                    FreeRtos::delay_ms(
                        500
                    );



                    wifi.start()?;



                    FreeRtos::delay_ms(
                        500
                    );



                    wifi.set_configuration(
                        &wifi_config()
                    )?;


                }
                else
                {


                    let _=
                        wifi.disconnect();



                    FreeRtos::delay_ms(
                        1000
                    );

                }


            }


        }


    }



    if !connected
    {

        return Err(
            anyhow::anyhow!(
                "wifi connect failed"
            )
        );

    }



    /*
        DHCP
    */


    wifi.wait_netif_up()?;



    let ip =
        wifi
        .wifi()
        .sta_netif()
        .get_ip_info()?
        .ip;



    log::info!(
        "wifi ip={}",
        ip
    );



    /*
        保持wifi

        当前架构需要
    */

    core::mem::forget(
        wifi
    );


    Ok(ip)

}