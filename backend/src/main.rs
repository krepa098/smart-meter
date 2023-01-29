use std::sync::{Arc, Mutex};

use actix_web::rt::net::UdpSocket;
use anyhow::Result;
use protocol::wire::{dgram::Pipeline, middleware};
use tokio::signal;

mod db;
mod packet;
mod schema;
mod web;

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
                    // let packet = ciborium::de::from_reader::<packet::Packet, &[u8]>(&buf);

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
                                        v_bat: None,
                                    })
                                    .unwrap();
                                }
                            }
                            _ => (),
                        }
                        dbg!(packet);
                    }

                }
                Ok(()) = signal::ctrl_c() => { break; }
            }
        }
    });

    let _ = tokio::join!(web::new_http_server(web_db), task);
    Ok(())
}
