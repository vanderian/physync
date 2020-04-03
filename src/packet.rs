pub use packet_struct::Packet;
pub use enums::PacketType;
pub use outgoing::{OutgoingPacket, OutgoingPacketBuilder};
pub use packet_reader::PacketReader;

pub mod header;

mod packet_struct;
mod enums;
mod outgoing;
mod packet_reader;

pub trait EnumConverter {
    type Enum;

    fn to_u8(&self) -> u8;
}
