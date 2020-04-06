use std::net::SocketAddr;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Packet {
    /// The endpoint from where it came.
    addr: SocketAddr,
    /// The raw payload of the packet.
    payload: Box<[u8]>,
}

impl Packet {
    /// Creates a new packet by passing the receiver and data
    pub fn new(addr: SocketAddr, payload: Box<[u8]>) -> Packet {
        Packet { addr, payload }
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
