use std::net::SocketAddr;

use rand::random;
use log::debug;

use crate::errors::Result;
use crate::features::connectivity::ConnectivityState::{Connected, Disconnected, Pending};
use crate::packet::header::{BaseHeader, SessionHeader};
use crate::packet::PacketType;
use crate::{ErrorKind, OutgoingPacketBuilder, Packet};

#[derive(PartialEq)]
enum ConnectivityState {
    Pending,
    Connected,
    Disconnected,
}

pub struct ConnectivityHandler {
    state: ConnectivityState,
    id: u64,
    peer_id: Option<u64>,
}

impl ConnectivityHandler {
    pub fn new() -> Self {
        ConnectivityHandler {
            state: ConnectivityState::Pending,
            id: random(),
            peer_id: None,
        }
    }

    pub fn process_in(
        &mut self,
        header: &BaseHeader,
        session: SessionHeader,
        peer_id: Option<SessionHeader>,
    ) -> Result<()> {
        // initial request, save peer id, `update` dispatches challenge response
        if self.peer_id.is_none() && peer_id.is_some() {
            self.peer_id = Some(peer_id.unwrap().session_id());
            return Ok(());
        }
        // peer should have a valid session by now
        self.check_session(&session)?;
        // challenge response request
        if self.state == Pending {
            debug!("connected!");
            self.state = Connected;
        }
        if header.packet_type() == PacketType::Disconnect {
            debug!("disconnected!");
            self.state = Disconnected;
        }

        Ok(())
    }

    pub fn session_id(&self) -> u64 {
        self.peer_id.map(|id| id ^ self.id).unwrap_or(0)
    }

    pub fn create_connection_packet(&self, addr: SocketAddr) -> Option<Packet> {
        // connection request, connection request response
        if self.state == Pending {
            let out = OutgoingPacketBuilder::new(&[])
                .with_session_header(self.id)
                .build();
            return Some(Packet::new(addr, out.contents()));
        }

        None
    }

    pub fn should_drop(&self) -> bool {
        self.state == Disconnected
    }

    pub fn is_connected(&self) -> bool {
        self.state == Connected
    }

    fn check_session(&self, session: &SessionHeader) -> Result<()> {
        if session.session_id() != self.session_id() {
            return Err(ErrorKind::SessionMismatch);
        }
        Ok(())
    }
}

impl Default for ConnectivityHandler {
    fn default() -> Self {
        Self::new()
    }
}
