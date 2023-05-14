use anyhow::{bail, Result};
use std::{net::Ipv4Addr, time::Duration};

use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    netif::{EspNetif, EspNetifWait},
    wifi::{EspWifi, WifiWait},
};
use heapless::String;

#[derive(Debug)]
pub struct Credential {
    pub ssid: String<32>,
    pub pw: String<64>,
}

pub struct WiFi {
    credentials: Vec<Credential>,
    wifi: Box<EspWifi<'static>>,
    sys_loop: EspSystemEventLoop,
    ssid: Option<String<32>>,

    // cached
    cached_channel: Option<u8>,
    cached_bssid: Option<[u8; 6]>,
    cached_strongest_signal: Option<usize>,
}

impl WiFi {
    pub fn new(
        modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
        sys_loop: EspSystemEventLoop,
        credentials: Vec<Credential>,
    ) -> Result<Self> {
        Ok(Self {
            credentials,
            wifi: Box::new(EspWifi::new(modem, sys_loop.clone(), None)?),
            sys_loop,
            ssid: None,
            cached_bssid: None,
            cached_channel: None,
            cached_strongest_signal: None,
        })
    }

    pub fn stop(&mut self) -> Result<()> {
        self.wifi.stop()?;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        if self.cached_strongest_signal.is_none() {
            self.scan()?;
        }

        if let Some(ss_id) = self.cached_strongest_signal {
            let credential = &self.credentials[ss_id];

            let client_config = ClientConfiguration {
                ssid: credential.ssid.to_owned(),
                auth_method: embedded_svc::wifi::AuthMethod::WPA2Personal,
                password: credential.pw.clone(),
                channel: self.cached_channel,
                bssid: self.cached_bssid,
            };
            self.ssid = Some(credential.ssid.to_owned());
            self.wifi
                .set_configuration(&Configuration::Client(client_config))?;

            self.wifi.start()?;
            if !WifiWait::new(&self.sys_loop)?
                .wait_with_timeout(Duration::from_secs(20), || self.wifi.is_started().unwrap())
            {
                bail!("Wifi did not start");
            }
        }

        Ok(())
    }

    pub fn scan(&mut self) -> Result<()> {
        let info = self.wifi.scan()?;
        log::info!("WiFi networks:");
        for (i, fo) in info.iter().enumerate() {
            log::info!("\t{}| '{}' ({} db)", i + 1, fo.ssid, fo.signal_strength);

            // check if we have credentials for this network
            if self.cached_strongest_signal.is_none() {
                if let Some((i, c)) = self
                    .credentials
                    .iter()
                    .enumerate()
                    .find(|(_, c)| c.ssid == fo.ssid)
                {
                    self.cached_strongest_signal = Some(i);
                    log::info!("Preferred wifi network: '{}'", c.ssid)
                }
            }
        }

        Ok(())
    }

    pub fn connect_blocking(&mut self) -> Result<()> {
        if !self.is_started() || self.is_connected() {
            return Ok(());
        }

        let mut retries = 3;
        while retries > 0 {
            log::info!("Connecting...");
            self.wifi.connect()?;

            retries -= 1;
            if EspNetifWait::new::<EspNetif>(self.wifi.sta_netif(), &self.sys_loop)?
                .wait_with_timeout(Duration::from_secs(10), || {
                    self.wifi.is_up().unwrap()
                        && self.wifi.sta_netif().get_ip_info().unwrap().ip
                            != Ipv4Addr::new(0, 0, 0, 0)
                })
            {
                break;
            } else if retries == 0 {
                bail!("Wifi did not connect or did not receive a DHCP lease");
            }

            std::thread::sleep(Duration::from_secs(5));
        }

        // cache for faster reconnect
        self.cached_bssid = self
            .wifi
            .driver()
            .get_configuration()
            .unwrap()
            .as_client_conf_ref()
            .unwrap()
            .bssid;

        self.cached_channel = self
            .wifi
            .driver()
            .get_configuration()
            .unwrap()
            .as_client_conf_ref()
            .unwrap()
            .channel;

        Ok(())
    }

    pub fn connect(&mut self) -> Result<()> {
        if !self.is_started() || self.is_connected() {
            return Ok(());
        }

        log::info!("Connecting...");
        self.wifi.connect()?;

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        if let Ok(ip_info) = self.wifi.sta_netif().get_ip_info() {
            return ip_info.ip != Ipv4Addr::new(0, 0, 0, 0);
        }
        false
    }

    pub fn is_started(&self) -> bool {
        self.wifi.is_started().unwrap_or(false)
    }

    pub fn ssid(&self) -> &Option<String<32>> {
        &self.ssid
    }
}
