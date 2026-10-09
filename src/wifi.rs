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

            ssid: WIFI_SSID
                .try_into()
                .unwrap(),

            password: WIFI_PASS
                .try_into()
                .unwrap(),

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



    /*
        启动 WiFi 驱动
    */

    wifi.start()?;


    log::info!(
        "wifi driver start"
    );


    /*
        等待 PHY 稳定

        ESP32-S3 建议保留
    */

    FreeRtos::delay_ms(1000);



    wifi.set_configuration(
        &wifi_config()
    )?;



    /*
        WiFi 连接重试
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

                    防止 ESP32-S3
                    第一次关联失败后卡死
                */


                let _ =
                    wifi.disconnect();



                let _ =
                    wifi.stop();



                /*
                    等待 WiFi driver 完全停止
                */

                FreeRtos::delay_ms(1000);



                /*
                    重新启动 WiFi
                */

                wifi.start()?;



                /*
                    等待 driver ready
                */

                FreeRtos::delay_ms(300);



                wifi.set_configuration(
                    &wifi_config()
                )?;



                /*
                    给扫描/关联准备时间
                */

                FreeRtos::delay_ms(500);


            }

        }



        if retry == 3 {


            anyhow::bail!(
                "wifi connect failed"
            );


        }


    }



    /*
        等待 DHCP 获取 IP
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
        保持 WiFi 生命周期

        不允许 Drop
    */

    core::mem::forget(wifi);



    Ok(ip)

}