use std::net::SocketAddr;
use crate::packet::enums::PacketType;
use crate::packet::enums::PacketType::{Data, Connect};

#[derive(Clone, Debug)]
pub struct Packet {
    /// The type of packet
    packet_type: PacketType,
    /// The endpoint from where it came.
    addr: SocketAddr,
    /// The raw payload of the packet.
    payload: Box<[u8]>,
}

impl Packet {
    /// Creates a new packet by passing the receiver and data
    // pub(crate) fn new(addr: SocketAddr, payload: Box<[u8]>) -> Packet {
    //     Packet { addr, payload }
    // }

    /// Creates a new unreliable packet by passing the receiver and data.
    pub fn data(addr: SocketAddr, payload: Vec<u8>) -> Packet {
        Packet {
            packet_type: Data,
            addr,
            payload: payload.into_boxed_slice(),
        }
    }

    pub fn connect(addr: SocketAddr) -> Packet {
        Packet {
            packet_type: Connect,
            addr,
            payload: vec![].into_boxed_slice(),
        }
    }

    pub fn packet_type(&self) -> PacketType {
        self.packet_type
    }

    /// Returns the payload of this packet.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Returns the address of this packet.
    ///
    /// # Remark
    /// Could be both the receiving endpoint or the one to send this packet to.
    /// This depends whether it is a packet that has been received or one that needs to be send.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
