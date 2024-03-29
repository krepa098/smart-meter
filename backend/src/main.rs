use std::sync::{Arc, Mutex};

use actix_web::rt::net::UdpSocket;
use anyhow::Result;
use common::packet::{Packet, Payload};
use dotenvy::dotenv;
use tokio::signal;

mod api;
mod db;
//mod req;
mod schema;
mod utils;

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    dotenv().ok();

    let sock = UdpSocket::bind("0.0.0.0:8989").await?;

    let db = Arc::new(Mutex::new(db::Db::connect()?));
    let web_db = db.clone();

    let task = actix_web::rt::spawn(async move {
        let mut buf = [0; 1024];
        println!("Listening...");
        loop {
            tokio::select! {
                Ok((len, _addr)) = sock.recv_from(&mut buf) => {
                    // try deserialize
                    let packet: postcard::Result<Packet> = postcard::from_bytes(&buf[0..len]);

                    if let Ok(packet) = packet {
                        let device_id = packet.header.device_id;

                        // calculate timestamp of the packet
                        // based on the offset relative to the time it was sent
                        // ignoring network latency
                        let timestamp = utils::utc_with_offset(packet.header.rel_timestamp);

                        match &packet.payload {
                            Payload::Measurement(mes) => {
                                if let Ok(mut db) = db.lock() {
                                    db.insert_measurement(&db::models::NewDeviceMeasurement {
                                        device_id: device_id as i32,
                                        timestamp: timestamp.timestamp_millis(),
                                        temperature: mes.temperature,
                                        pressure: mes.pressure,
                                        humidity: mes.humidity,
                                        air_quality: mes.air_quality,
                                        bat_v: mes.bat_voltage,
                                        bat_cap: mes.bat_capacity,
                                    })
                                    .unwrap();
                                }
                            }
                            Payload::DeviceInfo(info) => {
                                if let Ok(mut db) = db.lock() {
                                    db.update_device_info(&db::models::DeviceInfo {
                                        device_id: device_id as i32,
                                        fw_version: format!("{}.{}.{}.{}",
                                            info.firmware_version[0],
                                            info.firmware_version[1],
                                            info.firmware_version[2],
                                            info.firmware_version[3]),
                                        bsec_version: format!("{}.{}.{}.{}",
                                            info.bsec_version[0],
                                            info.bsec_version[1],
                                            info.bsec_version[2],
                                            info.bsec_version[3]),
                                        wifi_ssid: info.wifi_ssid.map(|b|std::str::from_utf8(&b).unwrap().to_owned()),
                                        uptime: info.uptime as i32,
                                        report_interval: info.report_interval as i32,
                                        sample_interval: info.sample_interval as i32,
                                        last_seen: utils::utc_with_offset(0).timestamp(),
                                    })
                                    .unwrap();
                                }
                                dbg!(info);
                            },
                        }
                    }

                }
                Ok(()) = signal::ctrl_c() => { break; }
            }
        }
    });

    let _ = tokio::join!(api::new_http_server(web_db), task);
    Ok(())
}
