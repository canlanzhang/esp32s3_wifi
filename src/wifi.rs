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


fn wifi_config() -> Configuration {

    Configuration::Client(
        ClientConfiguration {

            ssid: WIFI_SSID.try_into().unwrap(),

            password: WIFI_PASS.try_into().unwrap(),

            auth_method: AuthMethod::WPA2Personal,

            channel: None,

            ..Default::default()
        }
    )
}



pub fn connect_wifi(
    modem: esp_idf_hal::modem::Modem
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
                Some(nvs)
            )?,
            sysloop
        )?;



    wifi.start()?;


    log::info!("wifi driver start");


    /*
        给 PHY 稳定时间

        这里不要超过1秒
    */

    FreeRtos::delay_ms(1000);



    wifi.set_configuration(
        &wifi_config()
    )?;



    /*
        第一次连接
    */

    for retry in 1..=3 {


        log::info!(
            "wifi connect try {}",
            retry
        );


        match wifi.connect()
        {


            Ok(_) => {

                log::info!(
                    "wifi connected"
                );

                break;
            }



            Err(e) => {


                log::warn!(
                    "wifi connect fail {:?}",
                    e
                );


                /*
                    清理 WiFi 状态

                    关键部分
                */


                let _ =
                    wifi.disconnect();



                let _ =
                    wifi.stop();



                FreeRtos::delay_ms(300);



                wifi.start()?;



                wifi.set_configuration(
                    &wifi_config()
                )?;


                FreeRtos::delay_ms(200);


            }

        }


        if retry == 3 {

            anyhow::bail!(
                "wifi connect failed"
            );

        }

    }



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
        保留wifi生命周期
    */

    core::mem::forget(wifi);



    Ok(ip)

}