use anyhow::Result;
use std::net::UdpSocket;

use common::packet::Packet;

pub struct Client {
    socket: UdpSocket, // 508 bytes (single fragment)
    command_socket: UdpSocket,
    queue: heapless::Vec<Packet, 40>,
}

impl Client {
    pub fn new() -> Result<Self> {
        // x.x.x.255?

        let socket = UdpSocket::bind("0.0.0.0:8989")?;
        socket.connect("192.168.178.255:8989")?;
        socket.set_broadcast(true)?;

        let command_socket = UdpSocket::bind("192.168.178.255:6464")?;

        Ok(Self {
            socket,
            command_socket,
            queue: heapless::Vec::new(),
        })
    }

    pub fn broadcast_pkt(&self, payload: &Packet) -> Result<()> {
        let buffer: heapless::Vec<u8, 256> = postcard::to_vec(payload)?;
        self.socket.send(&buffer)?;

        Ok(())
    }

    pub fn broadcast_queue(&mut self) -> Result<()> {
        self.timestamp_to_rel();

        for pkt in &self.queue {
            self.broadcast_pkt(pkt)?;
        }
        self.queue.clear();

        Ok(())
    }

    fn timestamp_to_rel(&mut self) {
        let last_timestamp = self.queue.last().map_or(0, |pkg| pkg.header.timestamp);
        for pkt in &mut self.queue {
            pkt.header.timestamp = pkt.header.timestamp.saturating_sub(last_timestamp);
        }
    }

    pub fn enqueue(&mut self, pkt: Packet) -> Result<()> {
        if let Err(err) = self.queue.push(pkt) {
            self.queue.clear();
            log::warn!("Broadcast queue cleared. Reason: {:?}", err);
            // TODO: error
        };
        Ok(())
    }
}
