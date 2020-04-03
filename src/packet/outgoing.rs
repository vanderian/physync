use crate::packet::header::{BaseHeader, HeaderWriter, SessionHeader};
use crate::packet::PacketType;

/// Builder that could be used to construct an outgoing packet.
pub struct OutgoingPacketBuilder<'p> {
    header: Vec<u8>,
    payload: &'p [u8],
}

impl<'p> OutgoingPacketBuilder<'p> {
    /// Construct a new builder from the given `payload`.
    pub fn new(payload: &'p [u8]) -> OutgoingPacketBuilder<'p> {
        OutgoingPacketBuilder {
            header: Vec::new(),
            payload,
        }
    }

    /// Adds the `SessionHeader` to the header.
    pub fn with_session_header(mut self, session_id: u64) -> Self {
        let header = SessionHeader::new(session_id);

        header
            .parse(&mut self.header)
            .expect("Could not write session header to buffer");

        self
    }

    /// Adds the [`BaseHeader`](./header/base_header) to the header.
    pub fn with_default_header(mut self, packet_type: PacketType) -> Self {
        let header = BaseHeader::new(packet_type);
        header
            .parse(&mut self.header)
            .expect("Could not write default header to buffer");

        self
    }

    /// Constructs an `OutgoingPacket` from the contents constructed with this builder.
    pub fn build(self) -> OutgoingPacket<'p> {
        OutgoingPacket {
            header: self.header,
            payload: self.payload,
        }
    }
}

/// Packet that that contains data which is ready to be sent to a remote endpoint.
#[derive(Debug)]
pub struct OutgoingPacket<'p> {
    header: Vec<u8>,
    payload: &'p [u8],
}

impl<'p> OutgoingPacket<'p> {
    /// Return the contents of this packet; the content includes the header and payload bytes.
    ///
    /// # Remark
    /// - Until here we could use a reference to the outgoing data but here we need to do a hard copy.
    /// Because the header could vary in size but should be in front of the payload provided by the user.
    pub fn contents(&self) -> Box<[u8]> {
        [self.header.as_slice(), &self.payload]
            .concat()
            .into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{OutgoingPacketBuilder, PacketType};

    fn test_payload() -> Vec<u8> {
        b"test".to_vec()
    }

    #[test]
    fn assure_creation_session_header() {
        let payload = test_payload();

        let outgoing = OutgoingPacketBuilder::new(&payload)
            .with_session_header(1_u64)
            .build();

        let expected: Vec<u8> = [vec![0, 0, 0, 0, 0, 0, 0, 1], test_payload()]
            .concat()
            .to_vec();

        assert_eq!(outgoing.contents().to_vec(), expected);
    }

    #[test]
    fn assure_creation_default_header() {
        let payload = test_payload();

        let outgoing = OutgoingPacketBuilder::new(&payload)
            .with_default_header(
                PacketType::Connect,
            )
            .build();

        let expected: Vec<u8> = [vec![1], test_payload()].concat().to_vec();

        assert_eq!(
            outgoing.contents()[2..outgoing.contents().len()].to_vec(),
            expected
        );
    }
}
