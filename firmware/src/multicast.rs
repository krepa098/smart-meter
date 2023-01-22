use std::net::UdpSocket;

use anyhow::Result;

use crate::bme680;

const MAGIC: &str = "M1S1";

pub struct Client {
    socket: UdpSocket,
    command_socket: UdpSocket,
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
        })
    }

    pub fn send(&mut self) -> Result<()> {
        // limited to 64k
        self.socket.send("Hello World".as_bytes())?;
        Ok(())
    }

    pub fn broadcast<P: ?Sized + serde::Serialize>(&mut self, payload: &P) -> Result<()> {
        let mut buffer = [0u8; 256];
        ciborium::ser::into_writer(payload, buffer.as_mut_slice())?;

        let mut i = 0;
        for (bi, b) in buffer.iter().rev().enumerate() {
            if *b != 0 {
                i = bi - 1;
                break;
            }
        }

        println!("buf len {}", buffer.len() - i);
        self.socket.send(&buffer[0..buffer.len() - i])?;

        Ok(())
    }
}
