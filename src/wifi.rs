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

        ClientConfiguration {

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
                不固定频道。
            */
            channel:
                None,

            ..Default::default()
        }

    )
}



pub fn connect_wifi(
    modem: esp_idf_hal::modem::Modem
)
-> Result<std::net::Ipv4Addr>
{
    /*
        系统事件循环
    */
    let sysloop =
        EspSystemEventLoop::take()?;


    /*
        NVS
    */
    let nvs =
        EspDefaultNvsPartition::take()?;


    /*
        创建 WiFi
    */
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
        ==================================================
        第一次启动
        ==================================================
    */

    wifi.start()?;


    log::info!(
        "wifi driver started"
    );


    /*
        驱动刚启动。

        不需要等待 500ms / 800ms / 1500ms。

        100ms 足够让调用时序更稳定。
    */

    FreeRtos::delay_ms(
        100
    );



    /*
        设置 WiFi 配置
    */

    wifi.set_configuration(
        &wifi_config()
    )?;


    log::info!(
        "wifi configuration set"
    );



    /*
        ==================================================
        第一次连接
        ==================================================
    */

    log::info!(
        "wifi connect attempt 1"
    );


    match wifi.connect()
    {

        /*
            ==============================================
            第一次成功
            ==============================================
        */

        Ok(_) =>
        {
            log::info!(
                "wifi connected on first attempt"
            );
        }


        /*
            ==============================================
            第一次失败
            ==============================================
        */

        Err(e) =>
        {
            log::warn!(
                "wifi first connect failed: {:?}",
                e
            );


            /*
                ==================================================
                第一次失败直接重置 WiFi 驱动
                ==================================================
            */

            log::warn!(
                "reset wifi driver"
            );


            /*
                清除连接状态
            */

            let _ =
                wifi.disconnect();


            /*
                停止 WiFi 驱动
            */

            let _ =
                wifi.stop();


            /*
                给 ESP-IDF WiFi task 时间
                完成底层状态清理。

                300ms 足够。
            */

            FreeRtos::delay_ms(
                300
            );


            /*
                重新启动 WiFi
            */

            wifi.start()?;


            log::info!(
                "wifi driver restarted"
            );


            /*
                不需要等待 500~800ms。

                100ms 足够。
            */

            FreeRtos::delay_ms(
                100
            );


            /*
                stop/start 后重新配置。

                这一点非常重要。
            */

            wifi.set_configuration(
                &wifi_config()
            )?;


            /*
                ==================================================
                第二次连接
                ==================================================
            */

            log::info!(
                "wifi connect attempt 2"
            );


            wifi.connect()?;


            log::info!(
                "wifi connected after driver reset"
            );
        }
    }



    /*
        ==================================================
        DHCP
        ==================================================
    */

    wifi.wait_netif_up()?;


    /*
        获取 IP
    */

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
        ==================================================
        保持 WiFi 生命周期
        ==================================================
    */

    core::mem::forget(
        wifi
    );


    Ok(ip)
}