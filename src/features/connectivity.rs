use std::net::SocketAddr;

use log::debug;

use crate::{ErrorKind, OutgoingPacketBuilder, Packet};
use crate::errors::{DecodingErrorKind, Result};
use crate::errors::ErrorKind::DecodingError;
use crate::features::connectivity::ConnectivityState::{Connected, Disconnected, Pending};
use crate::net::constants::CONNECT_PAYLOAD_SIZE;
use crate::packet::{PacketReader, PacketType};
use crate::packet::header::{BaseHeader, SessionHeader};
use rand::random;

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
        reader: &mut PacketReader,
    ) -> Result<()> {
        let session = reader.read_session_header()?;

        if header.packet_type() == PacketType::Connect {
            let peer_id = reader.read_id_header()?;
            if !reader.can_read(CONNECT_PAYLOAD_SIZE) {
                return Err(DecodingError(DecodingErrorKind::Payload));
            }

            if self.peer_id.is_none() {
                self.peer_id = Some(peer_id.session_id());
                return Ok(());
            }
        }
        self.check_session(&session)?;

        if header.packet_type() == PacketType::Disconnect {
            self.disconnect();
        }

        Ok(())
    }

    pub fn session_id(&self) -> u64 {
        self.peer_id.map(|id| id ^ self.id).unwrap_or(0)
    }

    pub fn create_connection_packet(&self, addr: SocketAddr) -> Option<Packet> {
        // challenge request to client
        if self.state == Pending {
            let out = OutgoingPacketBuilder::new(&[])
                .with_session_header(self.session_id())
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

    fn disconnect(&mut self) {
        debug!("disconnected!");
        self.state = Disconnected;
    }

    fn check_session(&mut self, session: &SessionHeader) -> Result<()> {
        if session.session_id() != self.session_id() {
            self.disconnect();
            return Err(ErrorKind::SessionMismatch);
        }
        // if we have a session set to connected
        if self.state == Pending {
            debug!("connected!");
            self.state = Connected;
        }
        Ok(())
    }
}

impl Default for ConnectivityHandler {
    fn default() -> Self {
        Self::new()
    }
}
