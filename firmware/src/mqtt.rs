use std::thread::JoinHandle;

use anyhow::Result;
use embedded_svc::{
    mqtt::client::{Connection, MessageImpl, QoS},
    utils::mqtt::client::ConnState,
};
use esp_idf_svc::mdns::EspMdns;
use esp_idf_svc::mqtt::client::*;
use esp_idf_sys::EspError;
use log::info;

// https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs

pub struct Client {
    client: EspMqttClient<ConnState<MessageImpl, EspError>>,
    thread_handle: JoinHandle<()>,
}

impl Client {
    pub fn new() -> Result<Self> {
        let client_config = MqttClientConfiguration {
            protocol_version: Some(MqttProtocolVersion::V3_1_1),
            client_id: Some("M1S1"),
            ..Default::default()
        };

        let (client, mut conn) =
            EspMqttClient::new_with_conn("udp://224.0.0.1:12345", &client_config)?;

        info!("Created MQTT client");

        // spawn thread that handles messages
        let thread_handle = std::thread::spawn(move || {
            while let Some(msg) = conn.next() {
                match msg {
                    Err(e) => info!("MQTT Message error: {}", e),
                    Ok(msg) => info!("Got MQTT Message {:?}", msg),
                }
            }
        });

        Ok(Self {
            client,
            thread_handle,
        })
    }

    // pub fn publish<P: ?Sized + serde::Serialize>(
    //     &mut self,
    //     topic: &str,
    //     payload: &P,
    // ) -> Result<()> {
    //     let payload_data = serde_json::to_string(payload)?;

    //     self.client
    //         .publish(topic, QoS::ExactlyOnce, false, payload_data.as_bytes())?;

    //     Ok(())
    // }
}
