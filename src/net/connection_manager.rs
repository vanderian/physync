use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;

use log::{debug, error};

use crate::errors::Result;
use crate::net::constants::DEFAULT_MTU;
use crate::net::{Connection, Socket};
use crate::packet::PacketType;
use crate::Packet;
use std::time::Instant;

// would be nicer to have a trait dependency on socket impl, but traits does not support async
#[derive(Debug)]
pub struct ConnectionManager {
    connections: HashMap<SocketAddr, Connection>,
    buffer: Vec<u8>,
    socket: Socket,
}

impl ConnectionManager {
    pub fn new(socket: Socket) -> Self {
        ConnectionManager {
            connections: HashMap::new(),
            buffer: vec![0; DEFAULT_MTU as usize],
            socket,
        }
    }

    /// Poll one read/write cycle
    pub async fn manual_poll(&mut self, time: Instant) -> Result<()> {
        match self.socket.receive_packet(self.buffer.as_mut()).await {
            Ok((payload, peer)) => {
                debug!("************************{:?}", time.elapsed());

                let connection = self
                    .connections
                    .entry(peer)
                    .or_insert_with(|| Connection::new(peer, time));

                // resend data packets to other peers
                match connection.process_in(payload, time)? {
                    Some(packet) => self.push_to_all(packet, time).await?,
                    _ => (),
                }
            }
            Err(e) => error!("encountered read socket error: {}", e),
        }

        // update all connections
        for con in self.connections.values_mut() {
            if let Some(packet) = con.update(time) {
                debug!("send on update: {:?}", packet);
                self.socket
                    .send_packet(&packet.addr(), packet.payload())
                    .await?;
            }
        }

        // iterate through all connections and remove those that should be dropped
        self.connections.retain(|_, con| !con.should_drop(time));

        Ok(())
    }

    /// Relay incoming data all other peers
    async fn push_to_all(&mut self, packet: Packet, time: Instant) -> Result<()> {
        let outgoing = self
            .connections
            .values_mut()
            // filter send to self
            .filter(|con| con.is_ready(&packet.addr()))
            .map(|con| con.process_out(&packet, PacketType::Data, time))
            .collect::<Vec<_>>();

        for p in outgoing {
            debug!("send relay: {:?}", packet);
            self.socket.send_packet(&p.addr(), p.payload()).await?;
        }

        Ok(())
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr()
    }
}
