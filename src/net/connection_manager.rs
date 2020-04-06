use std::collections::HashMap;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use log::{debug, error};
use tokio::time::interval_at;

use crate::errors::Result;
use crate::net::constants::DEFAULT_MTU;
use crate::net::{Connection, Socket};
use crate::packet::PacketType;
use crate::Packet;

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

    // temporary for server
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

    // temporary for client
    pub async fn connect(&mut self, addr: SocketAddr, time: Instant) -> Result<()> {
        let mut con = Connection::new(addr, time);
        let packet = con.update(time).unwrap();
        debug!("send client connect: {:?}", packet);
        self.socket.send_packet(&addr, packet.payload()).await?;
        self.connections.insert(addr, con);
        Ok(())
    }

    pub fn socket(&self) -> &Socket {
        &self.socket
    }

    pub async fn sending(&mut self, packet: &Packet) -> Result<()> {
        let mut i = interval_at(
            tokio::time::Instant::now() + Duration::from_secs(5),
            Duration::from_secs(2),
        );
        let con = self.connections.values_mut().last().unwrap();
        loop {
            i.tick().await;
            let p = con.process_out(packet, PacketType::Data, Instant::now());
            debug!("send client: {:?}", p);
            self.socket.send_packet(&p.addr(), p.payload()).await?;
        }
    }
}
