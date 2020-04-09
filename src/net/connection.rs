use std::fmt;
use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use crate::errors::Result;
use crate::features::ConnectivityHandler;
use crate::net::constants::{DEFAULT_HEARTBEAT, DEFAULT_IDLE_TIMEOUT};
use crate::packet::PacketReader;
use crate::packet::PacketType;
use crate::{ErrorKind, OutgoingPacketBuilder, Packet};

use log::debug;

pub struct Connection {
    last_seen: Instant,
    last_sent: Instant,
    peer_address: SocketAddr,

    connectivity: ConnectivityHandler,
}

impl Connection {
    pub fn new(peer_address: SocketAddr, time: Instant) -> Self {
        Connection {
            last_seen: time,
            last_sent: time,
            peer_address,
            connectivity: ConnectivityHandler::new(),
        }
    }

    /// Returns a [Duration] representing the interval since we last heard from the client
    pub fn last_seen(&self, time: Instant) -> Duration {
        time.saturating_duration_since(self.last_seen)
    }

    /// Returns a [Duration] representing the interval since we last sent to the client
    pub fn last_sent(&self, time: Instant) -> Duration {
        time.saturating_duration_since(self.last_sent)
    }

    pub fn process_in(&mut self, payload: &[u8], time: Instant) -> Result<Option<Packet>> {
        self.last_seen = time;

        let mut reader = PacketReader::new(payload);
        let header = reader.read_base_header()?;
        if !header.is_current_protocol() {
            return Err(ErrorKind::ProtocolVersionMismatch);
        }

        debug!(
            "incoming {:?} from {:?}",
            header.packet_type(),
            self.peer_address
        );

        self.connectivity.process_in(&header, &mut reader)?;

        if header.packet_type() == PacketType::Data {
            let payload = reader.read_payload();
            return Ok(Some(Packet::new(self.peer_address, payload)));
        }

        Ok(None)
    }

    pub fn process_out(&mut self, packet: &Packet, ptype: PacketType, time: Instant) -> Packet {
        self.last_sent = time;

        let out = OutgoingPacketBuilder::new(packet.payload())
            .with_default_header(ptype)
            .with_session_header(self.connectivity.session_id())
            .build();

        Packet::new(self.peer_address, out.contents())
    }

    pub fn update(&mut self, time: Instant) -> Option<Packet> {
        debug!(
            "last seen {:?}, last sent {:?} @{:?}",
            self.last_seen(time),
            self.last_sent(time),
            self
        );
        if let Some(connect) = self
            .connectivity
            .create_connection_packet(self.peer_address)
        {
            debug!("connect!");
            return Some(self.process_out(&connect, PacketType::Connect, time));
        } else if self.last_sent(time) >= DEFAULT_HEARTBEAT {
            debug!("heartbeat!");
            let heartbeat = Packet::new(self.peer_address, Box::default());
            return Some(self.process_out(&heartbeat, PacketType::Heartbeat, time));
        }

        None
    }

    pub fn should_drop(&self, time: Instant) -> bool {
        let drop = self.last_seen(time) >= DEFAULT_IDLE_TIMEOUT || self.connectivity.should_drop();
        if drop {
            debug!(
                "should drop {:?} last seen: {:?}",
                self,
                self.last_seen(time)
            );
        }
        drop
    }

    pub fn is_ready(&self, sender: &SocketAddr) -> bool {
        self.connectivity.is_connected() && *sender != self.peer_address
    }
}

impl Debug for Connection {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.peer_address.ip(), self.peer_address.port())
    }
}
