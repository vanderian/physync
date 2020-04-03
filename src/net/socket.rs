use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use tokio::net::{ToSocketAddrs, UdpSocket};

use crate::errors::Result;
use crate::net::constants::DEFAULT_MTU;

#[derive(Debug)]
pub struct Socket {
    receive_buffer: Vec<u8>,
    socket: UdpSocket,
}

impl Socket {
    pub async fn bind<A: ToSocketAddrs>(addresses: A) -> Result<Self> {
        let socket = UdpSocket::bind(addresses).await?;
        Self::bind_internal(socket)
    }

    pub async fn bind_any() -> Result<Self> {
        let loopback = Ipv4Addr::new(127, 0, 0, 1);
        let address = SocketAddrV4::new(loopback, 0);
        let socket = UdpSocket::bind(address).await?;
        Self::bind_internal(socket)
    }

    fn bind_internal(socket: UdpSocket) -> Result<Self> {
        Ok(Self {
            receive_buffer: vec![0; DEFAULT_MTU as usize],
            socket,
        })
    }

    /// Returns the local socket address
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
    }

    pub fn socket(self) -> UdpSocket {
        self.socket
    }

    /*
    /// Sends a single packet
    pub async fn send(&mut self, packet: Packet) -> Result<usize> {
        // let mut sock = &self.socket;
        Ok(self.socket.send_to(packet.payload(), packet.addr()).await?)
    }

    */
    /// Receives a single packet
    pub async fn recv(&mut self) -> Result<(SocketAddr, &[u8])> {
        let (size, addr) = self.socket.recv_from(&mut self.receive_buffer).await?;
        Ok((addr, &self.receive_buffer[..size]))
    }
}