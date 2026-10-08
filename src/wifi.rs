use anyhow::Result;


use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;


use esp_idf_svc::wifi::{
    AuthMethod,
    BlockingWifi,
    ClientConfiguration,
    Configuration,
    EspWifi,
};



const WIFI_SSID: &str =
    "HUAWEI-LL";


const WIFI_PASS: &str =
    "passworld";





pub fn connect_wifi(

    modem:
        esp_idf_hal::modem::Modem,

)
-> Result<std::net::Ipv4Addr>
{



    let sysloop =
        EspSystemEventLoop::take()?;



    let nvs =
        EspDefaultNvsPartition::take()?;




    let wifi =
        EspWifi::new(
            modem,
            sysloop.clone(),
            Some(nvs),
        )?;





    let mut wifi =
        BlockingWifi::wrap(
            wifi,
            sysloop,
        )?;






    let config =
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



                ..Default::default()

            }

        );






    wifi.set_configuration(
        &config
    )?;





    wifi.start()?;


    log::info!(
        "wifi started"
    );





    wifi.connect()?;


    log::info!(
        "wifi connecting"
    );





    wifi.wait_netif_up()?;





    let ip =
        wifi.wifi()
        .sta_netif()
        .get_ip_info()?
        .ip;





    log::info!(
        "wifi connected ip={}",
        ip
    );





    // 保持wifi存活
    core::mem::forget(wifi);





    Ok(ip)

}