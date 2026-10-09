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



const WIFI_SSID: &str = "HUAWEI-LL";

const WIFI_PASS: &str = "passworld";



fn wifi_config() -> Configuration
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
-> Result<std::net::Ipv4Addr>
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




    wifi.start()?;


    log::info!(
        "wifi driver start"
    );



    /*
        给 RF 初始化时间
    */

    FreeRtos::delay_ms(
        300
    );



    wifi.set_configuration(
        &wifi_config()
    )?;




    /*
        第一次连接
    */


    log::info!(
        "wifi connect try 1"
    );


    match wifi.connect()
    {

        Ok(_)=>{

            log::info!(
                "wifi connected"
            );

        }


        Err(e)=>{

            log::warn!(
                "first connect fail {:?}",
                e
            );


            /*
                ESP32-S3 常见启动竞态
                重启 driver
            */


            let _ =
                wifi.disconnect();


            let _ =
                wifi.stop();



            FreeRtos::delay_ms(
                300
            );



            wifi.start()?;



            FreeRtos::delay_ms(
                300
            );



            wifi.set_configuration(
                &wifi_config()
            )?;



            log::info!(
                "wifi connect try 2"
            );


            wifi.connect()?;


            log::info!(
                "wifi connected after restart"
            );

        }

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
        保留生命周期
    */

    core::mem::forget(
        wifi
    );



    Ok(ip)

}