use std::convert::TryFrom;

use crate::ErrorKind;
use crate::errors::DecodingErrorKind;
use crate::packet::EnumConverter;

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
/// Id to identify a certain packet type.
pub enum PacketType {
    Data = 0,
    Connect = 1,
    Disconnect = 2,
    Heartbeat = 3,
}

impl EnumConverter for PacketType {
    type Enum = PacketType;

    fn to_u8(&self) -> u8 {
        *self as u8
    }
}

impl TryFrom<u8> for PacketType {
    type Error = ErrorKind;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PacketType::Data),
            1 => Ok(PacketType::Connect),
            2 => Ok(PacketType::Disconnect),
            3 => Ok(PacketType::Heartbeat),
            _ => Err(ErrorKind::DecodingError(DecodingErrorKind::PacketType)),
        }
    }
}
