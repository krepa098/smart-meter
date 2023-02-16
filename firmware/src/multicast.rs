use anyhow::Result;
use protocol::wire::{dgram::Pipeline, middleware};
use std::{net::UdpSocket, sync::Mutex};

use crate::packet::Packet;

pub struct Client {
    socket: UdpSocket, // 508 bytes (single fragment)
    command_socket: UdpSocket,
    pipeline: Mutex<Pipeline<Packet, middleware::pipeline::Default>>,
    queue: heapless::Vec<Packet, 40>,
}

impl Client {
    pub fn new() -> Result<Self> {
        // x.x.x.255?

        let socket = UdpSocket::bind("0.0.0.0:8989")?;
        socket.connect("192.168.178.255:8989")?;
        socket.set_broadcast(true)?;

        let command_socket = UdpSocket::bind("192.168.178.255:6464")?;

        let middleware = middleware::pipeline::default();
        let pipeline = Pipeline::new(
            middleware,
            protocol::Settings {
                byte_order: protocol::ByteOrder::LittleEndian,
            },
        );

        Ok(Self {
            socket,
            command_socket,
            pipeline: std::sync::Mutex::new(pipeline),
            queue: heapless::Vec::new(),
        })
    }

    pub fn broadcast_pkt(&self, payload: &Packet) -> Result<()> {
        let buffer = [0u8; 256];
        let mut cursor = std::io::Cursor::new(buffer);
        self.pipeline
            .lock()
            .unwrap()
            .send_to(&mut cursor, payload)
            .unwrap();

        let len = cursor.position() as usize;
        self.socket.send(&cursor.into_inner()[0..len])?;

        Ok(())
    }

    pub fn broadcast_queue(&mut self) -> Result<()> {
        for pkt in &self.queue {
            self.broadcast_pkt(pkt)?;
        }
        self.queue.clear();

        Ok(())
    }

    pub fn enqueue(&mut self, pkt: Packet) -> Result<()> {
        if let Err(_) = self.queue.push(pkt) {
            self.queue.clear();
            // TODO: error
        };
        Ok(())
    }
}
