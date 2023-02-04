use std::sync::{Arc, Mutex};

use actix_web::rt::net::UdpSocket;
use anyhow::Result;
use protocol::wire::{dgram::Pipeline, middleware};
use tokio::signal;

mod db;
mod packet;
mod schema;
mod utils;
mod web;

fn ms_since_epoch() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

#[actix_web::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let middleware = middleware::pipeline::default();
    let mut pipeline: Pipeline<packet::Packet, middleware::pipeline::Default> = Pipeline::new(
        middleware,
        protocol::Settings {
            byte_order: protocol::ByteOrder::LittleEndian,
        },
    );

    let sock = UdpSocket::bind("0.0.0.0:8989").await?;

    let db = Arc::new(Mutex::new(db::Db::connect()?));
    let web_db = db.clone();

    let task = actix_web::rt::spawn(async move {
        let mut buf = [0; 1024];
        println!("Listening...");
        loop {
            tokio::select! {
                Ok((len, _addr)) = sock.recv_from(&mut buf) => {
                    let mut data = std::io::Cursor::new(&buf[0..len]);
                    // try deserialize
                    let packet = pipeline.receive_from(&mut data);

                    if let Ok(packet) = packet {
                        let device_id = packet.header.device_id;

                        match &packet.payload {
                            packet::Payload::Measurement(mes) => {
                                if let Ok(mut db) = db.lock() {
                                    db.insert_measurement(&db::NewDeviceMeasurement {
                                        device_id: device_id as i32,
                                        timestamp: mes.timestamp as i64,
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
                            packet::Payload::DeviceInfo(info) => {
                                if let Ok(mut db) = db.lock() {
                                    db.update_device_info(&db::DeviceInfo {
                                        device_id: device_id as i32,
                                        fw_version: format!("v{}.{}.{}.{}",
                                            info.firmware_version[0],
                                            info.firmware_version[1],
                                            info.firmware_version[2],
                                            info.firmware_version[3]),
                                        bsec_version: format!("v{}.{}.{}.{}",
                                            info.bsec_version[0],
                                            info.bsec_version[1],
                                            info.bsec_version[2],
                                            info.bsec_version[3]),
                                        wifi_ssid: info.wifi_ssid.map(|b|std::str::from_utf8(&b).unwrap().to_owned()),
                                        uptime: info.uptime as i32,
                                        report_interval: info.report_interval as i32,
                                        sample_interval: info.sample_interval as i32,
                                        last_seen: (ms_since_epoch()/1000) as i64,
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

    let _ = tokio::join!(web::new_http_server(web_db), task);
    Ok(())
}
