use std::convert::TryFrom;
use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::errors::Result;
use crate::net::constants::BASE_HEADER_SIZE;
use crate::packet::enums::PacketType;
use crate::packet::header::header_reader::HeaderReader;
use crate::packet::header::header_writer::HeaderWriter;
use crate::packet::EnumConverter;
use crate::protocol_version::ProtocolVersion;

#[derive(Copy, Clone, Debug)]
/// This header will be included in each packet, and contains some basic information.
pub struct BaseHeader {
    protocol_version: u16,
    packet_type: PacketType,
}

impl BaseHeader {
    /// Creates new header.
    pub fn new(packet_type: PacketType) -> Self {
        BaseHeader {
            protocol_version: ProtocolVersion::get_crc16(),
            packet_type,
        }
    }

    /// Returns the protocol version
    #[cfg(test)]
    pub fn protocol_version(&self) -> u16 {
        self.protocol_version
    }

    /// Returns the PacketType
    pub fn packet_type(&self) -> PacketType {
        self.packet_type
    }

    /// Checks if the protocol version in the packet is a valid version
    pub fn is_current_protocol(&self) -> bool {
        ProtocolVersion::valid_version(self.protocol_version)
    }
}

impl Default for BaseHeader {
    fn default() -> Self {
        BaseHeader::new(PacketType::Data)
    }
}

impl HeaderWriter for BaseHeader {
    type Output = Result<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> Self::Output {
        buffer.write_u16::<BigEndian>(self.protocol_version)?;
        buffer.write_u8(self.packet_type.to_u8())?;
        Ok(())
    }
}

impl HeaderReader for BaseHeader {
    type Header = Result<BaseHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header {
        let protocol_version = rdr.read_u16::<BigEndian>()?; /* protocol id */
        let packet_id = rdr.read_u8()?;

        let header = BaseHeader {
            protocol_version,
            packet_type: PacketType::try_from(packet_id)?,
        };

        Ok(header)
    }

    /// Returns the size of this header.
    fn size() -> u8 {
        BASE_HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use byteorder::{BigEndian, ReadBytesExt};

    use crate::net::constants::BASE_HEADER_SIZE;
    use crate::packet::enums::PacketType;
    use crate::packet::header::{BaseHeader, HeaderReader, HeaderWriter};
    use crate::packet::EnumConverter;
    use crate::protocol_version::ProtocolVersion;

    #[test]
    fn serialize() {
        let mut buffer = Vec::new();
        let header = BaseHeader::new(PacketType::Data);
        assert![header.parse(&mut buffer).is_ok()];

        assert_eq!(
            buffer.as_slice().read_u16::<BigEndian>().unwrap(),
            ProtocolVersion::get_crc16()
        );
        assert_eq!(buffer[2], PacketType::Data.to_u8());
    }

    #[test]
    fn deserialize() {
        let buffer = vec![0, 1, 0];

        let mut cursor = Cursor::new(buffer.as_slice());

        let header = BaseHeader::read(&mut cursor).unwrap();

        assert_eq!(header.protocol_version(), 1);
        assert_eq!(header.packet_type(), PacketType::Data);
    }

    #[test]
    fn size() {
        assert_eq!(BaseHeader::size(), BASE_HEADER_SIZE);
    }
}
