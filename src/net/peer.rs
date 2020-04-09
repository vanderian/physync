use crate::errors::Result;
use crate::net::connection_manager::ConnectionManager;
use crate::net::Socket;
use log::error;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Instant;
use tokio::net::{ToSocketAddrs, UdpSocket};

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
        }
    }

    pub async fn manual_poll(&mut self, time: Instant) -> Result<()> {
        self.handler.manual_poll(time).await
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.handler.local_addr()
    }
}
