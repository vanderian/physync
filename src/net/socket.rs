use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::thread::yield_now;
use std::time::Instant;

use log::error;
use tokio::net::{ToSocketAddrs, UdpSocket};
use tokio::time::timeout;

use crate::errors::Result;
use crate::net::connection_manager::ConnectionManager;
use crate::net::constants::DEFAULT_IDLE_TIMEOUT;
use crate::Packet;

#[derive(Debug)]
pub struct Socket {
    pub socket: UdpSocket,
}

impl Socket {
    pub fn new(socket: UdpSocket) -> Self {
        Socket { socket }
    }

    pub async fn send_packet(&mut self, addr: &SocketAddr, payload: &[u8]) -> Result<usize> {
        Ok(self.socket.send_to(payload, addr).await?)
    }

    pub async fn receive_packet<'a>(
        &mut self,
        buffer: &'a mut [u8],
    ) -> Result<(&'a [u8], SocketAddr)> {
        let rx = timeout(DEFAULT_IDLE_TIMEOUT, self.socket.recv_from(buffer));
        Ok(rx
            .await?
            .map(move |(recv_len, address)| (&buffer[..recv_len], address))?)
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
    }
}

#[derive(Debug)]
pub struct Peer {
    handler: ConnectionManager,
}

impl Peer {
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
            handler: ConnectionManager::new(Socket::new(socket)),
        })
    }

    pub async fn in_loop(&mut self) {
        loop {
            match self.manual_poll(Instant::now()).await {
                Ok(()) => (),
                Err(e) => error!("encountered error: {}", e),
            };
            yield_now();
        }
    }

    pub async fn manual_poll(&mut self, time: Instant) -> Result<()> {
        self.handler.manual_poll(time).await
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.handler.socket().local_addr()
    }

    pub async fn connect(&mut self, addr: SocketAddr) -> Result<()> {
        self.handler.connect(addr, Instant::now()).await
    }

    pub async fn loop_send(&mut self, packet: &Packet) -> Result<()> {
        self.handler.sending(packet).await
    }
}
