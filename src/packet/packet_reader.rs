use std::io::Cursor;

use crate::{ErrorKind, Result};
use crate::packet::header::{BaseHeader, HeaderReader, SessionHeader};

/// Can be used to read the packet contents.
///
/// # Remarks
/// - `PacketReader` is using an underlying `Cursor` to manage the reading of the bytes.
/// - `PacketReader` can interpret where some data is located in the buffer, that's why you don't have to worry about the position of the `Cursor`.
pub struct PacketReader<'s> {
    buffer: &'s [u8],
    cursor: Cursor<&'s [u8]>,
}

impl<'s> PacketReader<'s> {
    /// Construct a new instance of `PacketReader`, the given `buffer` will be used to read information from.
    pub fn new(buffer: &'s [u8]) -> PacketReader<'s> {
        PacketReader {
            buffer,
            cursor: Cursor::new(buffer),
        }
    }

    /// Reads the `BaseHeader` from the underlying buffer.
    ///
    /// # Remark
    /// - Will change the position to the location of `BaseHeader`
    pub fn read_base_header(&mut self) -> Result<BaseHeader> {
        self.cursor.set_position(0);

        if self.can_read(BaseHeader::size()) {
            BaseHeader::read(&mut self.cursor)
        } else {
            Err(ErrorKind::CouldNotReadHeader(String::from("base")))
        }
    }

    /// Reads the `SessionHeader` for session id from the underlying buffer.
    ///
    /// # Remark
    /// - Will change the position to the location of `SessionHeader`
    pub fn read_session_header(&mut self) -> Result<SessionHeader> {
        self.session_header(u64::from(BaseHeader::size()), "session id")
    }

    /// Reads the `SessionHeader` for peer id from the underlying buffer.
    ///
    /// # Remark
    /// - Will change the position to the location of `SessionHeader`
    pub fn read_id_header(&mut self) -> Result<SessionHeader> {
        self.session_header(u64::from(BaseHeader::size() + SessionHeader::size()), "peer id")
    }

    fn session_header(&mut self, pos: u64, msg: &str) -> Result<SessionHeader> {
        self.cursor.set_position(pos);

        if self.can_read(SessionHeader::size()) {
            SessionHeader::read(&mut self.cursor)
        } else {
            Err(ErrorKind::CouldNotReadHeader(String::from(msg)))
        }
    }

    /// Reads the payload` from the underlying buffer.
    ///
    /// # Remark
    /// - Notice that this will continue on the position of last read header;
    /// e.g. when reading `BaseHeader` the position of the underlying `Cursor` will be at the end where it left of,
    /// when calling this function afterward it will read all the bytes from there on.
    pub fn read_payload(&self) -> Box<[u8]> {
        self.buffer[self.cursor.position() as usize..self.buffer.len()]
            .to_vec()
            .into_boxed_slice()
    }

    // Checks if a given length of bytes could be read with the buffer.
    fn can_read(&self, length: u8) -> bool {
        (self.buffer.len() - self.cursor.position() as usize) >= length as usize
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{PacketReader, PacketType};

    #[test]
    fn can_read_bytes() {
        let buffer = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let reader = PacketReader::new(buffer.as_slice());
        assert_eq!(reader.can_read(buffer.len() as u8), true);
        assert_eq!(reader.can_read((buffer.len() + 1) as u8), false);
    }

    #[test]
    fn assure_read_base_header() {
        // base header
        let payload: Vec<u8> = vec![vec![0, 1, 0]].concat();

        let mut reader = PacketReader::new(payload.as_slice());

        let header = reader.read_base_header().unwrap();

        assert_eq!(header.protocol_version(), 1);
        assert_eq!(header.packet_type(), PacketType::Data);
    }

    #[test]
    fn assure_read_session_header() {
        // base header, session header
        let payload: Vec<u8> =
            vec![vec![0, 1, 0], vec![0, 0, 0, 0, 0, 0, 0, 3]].concat();

        let mut reader = PacketReader::new(payload.as_slice());

        let header = reader.read_session_header().unwrap();

        assert_eq!(header.session_id(), 3);
    }

    #[test]
    fn assure_read_id_header() {
        // base header, session header, id header
        let payload: Vec<u8> =
            vec![vec![0, 1, 0], vec![0, 0, 0, 0, 0, 0, 0, 3], vec![0, 0, 0, 0, 0, 0, 0, 5]].concat();

        let mut reader = PacketReader::new(payload.as_slice());

        let header = reader.read_id_header().unwrap();

        assert_eq!(header.session_id(), 5);
    }

    #[test]
    fn expect_read_error() {
        // base header (with one corrupt byte)
        let payload: Vec<u8> = vec![vec![0, 1]].concat();

        let mut reader = PacketReader::new(payload.as_slice());

        assert!(reader.read_base_header().is_err());
    }
}
