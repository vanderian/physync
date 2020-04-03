use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use crate::errors::Result;
use crate::net::constants::SESSION_HEADER_SIZE;
use crate::packet::header::header_reader::HeaderReader;
use crate::packet::header::header_writer::HeaderWriter;

#[derive(Copy, Clone, Debug)]
/// This header will be included in each packet sent by client, with server_salt^client_salt value
///
/// When requesting connection intermediate values are used:
/// client provides client_salt, server provides server_salt with client_salt in payload,
pub struct SessionHeader {
    session_id: u64,
}

impl SessionHeader {
    /// Creates new header.
    pub fn new(session_id: u64) -> Self {
        SessionHeader { session_id }
    }

    /// Returns the PacketType
    pub fn session_id(&self) -> u64 {
        self.session_id
    }
}

impl HeaderWriter for SessionHeader {
    type Output = Result<()>;

    fn parse(&self, buffer: &mut Vec<u8>) -> Self::Output {
        buffer.write_u64::<BigEndian>(self.session_id)?;
        Ok(())
    }
}

impl HeaderReader for SessionHeader {
    type Header = Result<SessionHeader>;

    fn read(rdr: &mut Cursor<&[u8]>) -> Self::Header {
        let client_id = rdr.read_u64::<BigEndian>()?;

        let header = SessionHeader {
            session_id: client_id,
        };

        Ok(header)
    }

    /// Returns the size of this header.
    fn size() -> u8 {
        SESSION_HEADER_SIZE
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::net::constants::SESSION_HEADER_SIZE;
    use crate::packet::header::{HeaderReader, HeaderWriter, SessionHeader};

    #[test]
    fn serialize() {
        let mut buffer = Vec::new();
        let header = SessionHeader::new(1_u64);
        assert![header.parse(&mut buffer).is_ok()];

        assert_eq!(buffer[7], 1);
    }

    #[test]
    fn deserialize() {
        let buffer = vec![0, 0, 0, 0, 0, 0, 0, 1];

        let mut cursor = Cursor::new(buffer.as_slice());

        let header = SessionHeader::read(&mut cursor).unwrap();

        assert_eq!(header.session_id(), 1);
    }

    #[test]
    fn size() {
        assert_eq!(SessionHeader::size(), SESSION_HEADER_SIZE);
    }
}
