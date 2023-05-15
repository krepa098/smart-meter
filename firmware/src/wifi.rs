use anyhow::Result;
use std::time::Duration;

use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_hal::peripheral;
use esp_idf_svc::{eventloop::EspSystemEventLoop, wifi::BlockingWifi, wifi::EspWifi};
use heapless::String;

#[derive(Debug)]
pub struct Credential {
    pub ssid: String<32>,
    pub pw: String<64>,
}

pub struct WiFi {
    credentials: Vec<Credential>,
    wifi: Box<BlockingWifi<EspWifi<'static>>>,
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
        nvs: Option<esp_idf_svc::nvs::EspNvsPartition<esp_idf_svc::nvs::NvsDefault>>,
    ) -> Result<Self> {
        Ok(Self {
            credentials,
            wifi: Box::new(BlockingWifi::wrap(
                EspWifi::new(modem, sys_loop.clone(), nvs)?,
                sys_loop,
            )?),
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
        self.wifi
            .set_configuration(&Configuration::Client(ClientConfiguration::default()))?;
        self.wifi.start()?;

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
        }

        Ok(())
    }

    pub fn scan(&mut self) -> Result<()> {
        log::info!("WiFi networks:");
        let info = self.wifi.scan()?;
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

    pub fn connect(&mut self) -> Result<()> {
        if !self.is_started() || self.is_connected() {
            return Ok(());
        }

        for _ in 0..3 {
            log::info!("Connecting...");

            if self.wifi.connect().is_err() {
                log::warn!("Wifi connection failed... try again");
            } else {
                self.wifi.wait_netif_up()?;
                break;
            }

            std::thread::sleep(Duration::from_secs(5));
        }

        // cache for faster reconnect
        self.cached_bssid = self
            .wifi
            .wifi()
            .driver()
            .get_configuration()
            .unwrap()
            .as_client_conf_ref()
            .unwrap()
            .bssid;

        self.cached_channel = self
            .wifi
            .wifi()
            .driver()
            .get_configuration()
            .unwrap()
            .as_client_conf_ref()
            .unwrap()
            .channel;

        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.wifi.is_up().unwrap()
    }

    pub fn is_started(&self) -> bool {
        self.wifi.is_started().unwrap_or(false)
    }

    pub fn ssid(&self) -> &Option<String<32>> {
        &self.ssid
    }
}
