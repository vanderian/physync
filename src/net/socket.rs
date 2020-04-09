use std::net::SocketAddr;

use tokio::net::UdpSocket;
use tokio::time::timeout;

use crate::errors::Result;
use crate::net::constants::DEFAULT_IDLE_TIMEOUT;

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
