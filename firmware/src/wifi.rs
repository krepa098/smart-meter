use anyhow::{bail, Result};
use log::info;
use std::{net::Ipv4Addr, time::Duration};

use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    netif::{EspNetif, EspNetifWait},
    wifi::{EspWifi, WifiWait},
};
use heapless::String;

pub struct Credentials {
    pub ssid: String<32>,
    pub pw: String<64>,
}

pub struct WiFi {
    wifi: Box<EspWifi<'static>>,
    sys_loop: EspSystemEventLoop,
}

impl WiFi {
    pub fn new(
        modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
        sys_loop: EspSystemEventLoop,
    ) -> Result<Self> {
        Ok(Self {
            wifi: Box::new(EspWifi::new(modem, sys_loop.clone(), None)?),
            sys_loop,
        })
    }

    pub fn stop(&mut self) -> Result<()> {
        self.wifi.stop()?;
        Ok(())
    }

    pub fn start(&mut self, credentials: &Credentials) -> Result<()> {
        let client_config = ClientConfiguration {
            ssid: credentials.ssid.to_owned(),
            bssid: None, // TODO: cache to reconnect faster?
            auth_method: embedded_svc::wifi::AuthMethod::WPA2Personal,
            password: credentials.pw.clone(),
            channel: None, // TODO: cache to reconnect faster?
        };
        self.wifi
            .set_configuration(&Configuration::Client(client_config))?;

        self.wifi.start()?;
        if !WifiWait::new(&self.sys_loop)?
            .wait_with_timeout(Duration::from_secs(20), || self.wifi.is_started().unwrap())
        {
            bail!("Wifi did not start");
        }
        Ok(())
    }

    pub fn scan(&mut self) -> Result<()> {
        let info = self.wifi.scan()?;
        info!("WiFi networks:");
        for (i, fo) in info.iter().enumerate() {
            info!("\t{}| {} ({})", i + 1, fo.ssid, fo.signal_strength);
        }

        Ok(())
    }

    pub fn connect_blocking(&mut self) -> Result<()> {
        if !self.is_started()? || self.is_connected()? {
            return Ok(());
        }

        let mut retries = 3;
        while retries > 0 {
            info!("Connecting...");
            self.wifi.connect()?;

            retries -= 1;
            if EspNetifWait::new::<EspNetif>(self.wifi.sta_netif(), &self.sys_loop)?
                .wait_with_timeout(Duration::from_secs(10), || {
                    self.wifi.is_connected().unwrap()
                        && self.wifi.sta_netif().get_ip_info().unwrap().ip
                            != Ipv4Addr::new(0, 0, 0, 0)
                })
            {
                break;
            } else {
                if retries == 0 {
                    bail!("Wifi did not connect or did not receive a DHCP lease");
                }
            }

            std::thread::sleep(Duration::from_secs(5));
        }
        Ok(())
    }

    pub fn connect(&mut self) -> Result<()> {
        if !self.is_started()? || self.is_connected()? {
            return Ok(());
        }

        info!("Connecting...");
        self.wifi.connect()?;

        Ok(())
    }

    pub fn is_connected(&self) -> Result<bool> {
        Ok(self.wifi.sta_netif().get_ip_info()?.ip != Ipv4Addr::new(0, 0, 0, 0))
    }

    pub fn is_started(&self) -> Result<bool> {
        let ret = self.wifi.is_started()?;
        Ok(ret)
    }
}

// pub fn setup_wifi(
//     modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
//     sys_loop: EspSystemEventLoop,
//     ssid: &str,
//     pw: &str,
// ) -> Result<WiFi> {
//     let mut wifi = Box::new(EspWifi::new(modem, sys_loop.clone(), None)?);

//     info!("Scanning...");
//     let info = wifi.scan()?; // sorted by strength

//     info!("Wifis:");
//     for (i, fo) in info.iter().enumerate() {
//         info!("\t{}| {} ({})", i + 1, fo.ssid, fo.signal_strength);
//     }

//     let ap = info.iter().find(|a| a.ssid == ssid);

//     if let Some(ap) = ap {
//         let client_config = ClientConfiguration {
//             ssid: ap.ssid.to_owned(),
//             bssid: None,
//             auth_method: embedded_svc::wifi::AuthMethod::WPA2Personal,
//             password: String::from(pw),
//             channel: Some(ap.channel),
//             // ip_conf: Some(embedded_svc::ipv4::ClientConfiguration::DHCP(
//             //     DHCPClientSettings {
//             //         hostname: Some(String::from("esp32-c3")),
//             //     },
//             // )),
//         };
//         wifi.set_configuration(&Configuration::Client(client_config))?;

//         info!("Starting Wifi...");
//         wifi.start()?;

//         if !WifiWait::new(&sys_loop)?
//             .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
//         {
//             bail!("Wifi did not start");
//         }

//         // if !WifiWait::new(&sys_loop)?
//         //     .wait_with_timeout(Duration::from_secs(20), || !wifi.is_connected().unwrap())
//         // {
//         //     bail!("Cannot connect WiFi");
//         // }

//         let mut retries = 3;
//         while retries > 0 {
//             info!("Connecting with '{}'...", ssid);
//             wifi.connect()?;

//             retries -= 1;
//             if EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sys_loop)?.wait_with_timeout(
//                 Duration::from_secs(10),
//                 || {
//                     wifi.is_connected().unwrap()
//                         && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
//                 },
//             ) {
//                 break;
//             } else {
//                 if retries == 0 {
//                     bail!("Wifi did not connect or did not receive a DHCP lease");
//                 }
//             }

//             std::thread::sleep(Duration::from_secs(5));
//         }

//         let ip_info = wifi.sta_netif().get_ip_info()?;

//         info!("Wifi DHCP info: {:?}", ip_info);
//     }
//     Ok(WiFi {
//         wifi,
//         sys_loop,
//         ssid: ssid.into(),
//         pw: pw.into(),
//     })
//     // bail!("Wifi not found")
// }
