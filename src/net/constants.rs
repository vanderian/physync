/// The size of the standard header.
pub const BASE_HEADER_SIZE: u8 = 3;
/// The size of the client header.
pub const SESSION_HEADER_SIZE: u8 = 4;
/// Maximum transmission unit of the payload.
///
/// Derived from ethernet_mtu - ipv6_header_size - udp_header_size - packet header size
///       1452 = 1500         - 40               - 8               - 8
///
/// This is not strictly guaranteed -- there may be less room in an ethernet frame than this due to
/// variability in ipv6 header size.
pub const DEFAULT_MTU: u16 = 1452;
/// This is the current protocol version.
///
/// It is used for:
/// - Generating crc16 for the packet header.
/// - Validating if arriving packets have the same protocol version.
pub const PROTOCOL_VERSION: &str = "physync-0.1.0";