use std::{net::Ipv4Addr, time::Duration};

use anyhow::bail;
use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use log::info;

use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    netif::{EspNetif, EspNetifWait},
    wifi::{EspWifi, WifiWait},
};
use esp_idf_sys as _;
use heapless::String; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

pub fn setup_wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sys_loop: EspSystemEventLoop,
    ssid: &str,
    pw: &str,
) -> anyhow::Result<Box<EspWifi<'static>>> {
    let mut wifi = Box::new(EspWifi::new(modem, sys_loop.clone(), None)?);

    info!("Scanning...");
    let info = wifi.scan()?; // sorted by strength

    info!("Wifis:");
    for (i, fo) in info.iter().enumerate() {
        info!("\t{}| {} ({})", i + 1, fo.ssid, fo.signal_strength);
    }

    let ap = info.iter().find(|a| a.ssid == ssid);

    if let Some(ap) = ap {
        let client_config = ClientConfiguration {
            ssid: ap.ssid.to_owned(),
            bssid: None,
            auth_method: embedded_svc::wifi::AuthMethod::WPA2Personal,
            password: String::from(pw),
            channel: Some(ap.channel),
            // ip_conf: Some(embedded_svc::ipv4::ClientConfiguration::DHCP(
            //     DHCPClientSettings {
            //         hostname: Some(String::from("esp32-c3")),
            //     },
            // )),
        };
        wifi.set_configuration(&Configuration::Client(client_config))?;

        info!("Starting Wifi...");
        wifi.start()?;

        if !WifiWait::new(&sys_loop)?
            .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
        {
            bail!("Wifi did not start");
        }

        // if !WifiWait::new(&sys_loop)?
        //     .wait_with_timeout(Duration::from_secs(20), || !wifi.is_connected().unwrap())
        // {
        //     bail!("Cannot connect WiFi");
        // }

        let mut retries = 3;
        while retries > 0 {
            info!("Connecting with '{}'...", ssid);
            wifi.connect()?;

            retries -= 1;
            if EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sys_loop)?.wait_with_timeout(
                Duration::from_secs(10),
                || {
                    wifi.is_connected().unwrap()
                        && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
                },
            ) {
                break;
            } else {
                if retries == 0 {
                    bail!("Wifi did not connect or did not receive a DHCP lease");
                }
            }
        }

        let ip_info = wifi.sta_netif().get_ip_info()?;

        info!("Wifi DHCP info: {:?}", ip_info);
    }
    Ok(wifi)
    // bail!("Wifi not found")
}
